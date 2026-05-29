#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn modal_kind_mapping_matches_modal_variants() {
        assert_eq!(DialogModalType::Default.kind(), DialogModalKind::Default);
        assert_eq!(
            DialogModalType::Failure {
                error_code: "E-1".to_string(),
                confirm_label: "Confirm".to_string()
            }
            .kind(),
            DialogModalKind::Failure
        );
        assert_eq!(
            DialogModalType::Question {
                confirm_label: "Yes".to_string(),
                cancel_label: "No".to_string()
            }
            .kind(),
            DialogModalKind::Question
        );
        assert_eq!(DialogModalType::Blank.kind(), DialogModalKind::Blank);
    }

    #[test]
    fn dialog_constructors_set_expected_defaults() {
        let default = DialogConfig::default_modal("Title", "Body");
        assert_eq!(default.modal.kind(), DialogModalKind::Default);
        assert_eq!(default.provider, DialogProvider::BevyApp);

        let failure = DialogConfig::failure("Error", "Something failed", "E-500");
        assert_eq!(failure.modal.kind(), DialogModalKind::Failure);
        assert!(!failure.close_on_backdrop);

        let question = DialogConfig::question("Question", "Proceed?");
        assert_eq!(question.modal.kind(), DialogModalKind::Question);
        assert!(question.close_on_backdrop);

        let blank = DialogConfig::blank(DialogLayout::BottomSheet);
        assert_eq!(blank.modal.kind(), DialogModalKind::Blank);
        assert_eq!(blank.layout, DialogLayout::BottomSheet);
    }
}
