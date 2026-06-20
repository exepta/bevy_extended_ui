"use strict";

const RELEASE_RANK = {
  false: 0,
  patch: 1,
  minor: 2,
  major: 3,
};

const RANK_RELEASE = {
  1: "patch",
  2: "minor",
  3: "major",
};

const PATCH_BRANCH_RE = /^(?:fix|hotfix|bugfix|perf|refactor|build)\//;
const NO_RELEASE_BRANCH_RE = /^(?:docs|style|chore|test|ci)\//;
const MINOR_BRANCH_RE = /^feat\//;

function parseMergeSourceBranch(message) {
  const firstLine = String(message || "").split(/\r?\n/, 1)[0];
  const match = firstLine.match(/^Merge pull request #\d+ from [^/]+\/(.+)$/);
  return match ? match[1] : null;
}

function ruleRelease(rules, predicate, fallback) {
  const rule = (rules || []).find(predicate);
  return rule ? rule.release : fallback;
}

function conventionalReleaseType(message, rules) {
  const text = String(message || "");
  const header = text.split(/\r?\n/, 1)[0];
  const match = header.match(/^(\w+)(?:\(([^)]+)\))?(!)?:\s(.+)$/);

  if (/^BREAKING[ -]CHANGE:/m.test(text) || (match && match[3] === "!")) {
    return ruleRelease(rules, (rule) => rule.breaking === true, "major");
  }

  if (!match) {
    return false;
  }

  return ruleRelease(rules, (rule) => rule.type === match[1], false);
}

function highestReleaseType(commits, rules) {
  let rank = RELEASE_RANK.false;

  for (const commit of commits || []) {
    const releaseType = conventionalReleaseType(commit.message, rules);
    rank = Math.max(rank, RELEASE_RANK[releaseType] || RELEASE_RANK.false);
  }

  return RANK_RELEASE[rank] || false;
}

function betaBranchReleaseType(commits, rules) {
  const sourceBranches = (commits || [])
    .map((commit) => parseMergeSourceBranch(commit.message))
    .filter(Boolean);

  if (sourceBranches.length > 0) {
    if (sourceBranches.some((branch) => MINOR_BRANCH_RE.test(branch))) {
      return "minor";
    }
    if (sourceBranches.some((branch) => PATCH_BRANCH_RE.test(branch))) {
      return "patch";
    }
    if (sourceBranches.every((branch) => NO_RELEASE_BRANCH_RE.test(branch))) {
      return false;
    }
  }

  return highestReleaseType(commits, rules);
}

module.exports = {
  analyzeCommits: async (pluginConfig, context) => {
    const branchName = context.branch && context.branch.name;
    const commits = context.commits || [];
    const rules = pluginConfig.releaseRules || [];

    const releaseType =
      branchName === "beta"
        ? betaBranchReleaseType(commits, rules)
        : highestReleaseType(commits, rules);

    if (releaseType) {
      context.logger.log("Determined release type: %s", releaseType);
    } else {
      context.logger.log("No release type determined");
    }

    return releaseType;
  },
};
