Language Support

Overview
This crate can localize HTML content by replacing placeholders in your HTML with translated strings and runtime variables. Localization is only active when a language backend feature is enabled.

Enable A Backend
Choose one of the features in your Cargo.toml:
- fluent for Fluent (.ftl) files
- properties-lang for Java .properties files

You can enable both. When both are enabled, Fluent is tried first and Properties is used as a fallback.

Language Selection Order
The resolved language tag is chosen in this order:
1) Forced from the HTML tag <html lang="...">
2) Selected via the UILang resource
3) System language from the OS environment

If the HTML lang attribute is "auto" or "default", it does not force a language and the normal resolution continues.

Where Files Live
By default, language files are loaded from assets/lang. You can override the folder using ExtendedUiConfiguration.language_path.

File name lookup supports common tag formats. For a requested tag, the loader tries:
- the full tag in lowercase with hyphens
- a version with underscores
- region variants in upper/lower case
- the base language only

If no match is found, it falls back to English ("en").

Placeholder Syntax In HTML
Localization only runs on placeholders using double braces, like {{ KEY }}.

Inside the braces:
- Translation keys are plain tokens, such as {{ LANGUAGE_TITLE }}
- Variables use percent markers, such as {{ %player_name% }}
- You can mix both in one placeholder, separated by whitespace, for example: {{ WELCOME_START %player_name% WELCOME_END }}

If a translation key or variable is missing, the original token is kept.

Reactive Bindings
Reactive placeholders can coexist with localization.
Examples:
- {{ user.name }}
- {{ user.full_name() }}

If no translation key matches, these placeholders stay untouched so they can be consumed by the reactive widget attributes (`innerText`, `innerHtml`, `innerBindings`).

Fluent Backend (ftl)
Use .ftl files in the language folder. This implementation reads simple message values by key. Fluent arguments are not evaluated here, so use multiple keys plus %var% placeholders for runtime values.

Properties Backend (properties)
Use .properties files in the language folder. Lines are parsed as key/value pairs. Comments using # or ! are ignored. The same placeholder rules apply.

Runtime Updates
Localization is re-applied when:
- UILang changes (selected or forced)
- language_path changes
- UiLangVariables change

Set variables via the UiLangVariables resource; they are exposed as %var% tokens in placeholders.

Troubleshooting
- No {{ ... }} placeholders means no localization happens.
- Make sure the correct feature is enabled in Cargo.toml.
- Verify the language file name matches the resolved tag.
- The special system tags "c" and "posix" are ignored and treated as no system language.
