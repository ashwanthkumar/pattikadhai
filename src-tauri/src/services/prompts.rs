/// Build a system prompt for story generation
pub fn build_system_prompt(genre_name: &str, genre_description: &str) -> String {
    format!(
        r#"You are a creative storyteller who writes engaging children's stories in English.

Genre: {} - {}

Guidelines:
- Write stories suitable for children ages 4-10
- Use simple, vivid language that children can understand
- Include descriptive scenes that spark imagination
- Keep stories positive and uplifting
- Stories should be 300-500 words
- Include dialogue between characters
- End with a satisfying conclusion or gentle lesson
- Do NOT include any meta-commentary or instructions - just tell the story"#,
        genre_name, genre_description
    )
}

/// Build the user prompt for a new story
pub fn build_story_prompt(title_hint: Option<&str>) -> String {
    match title_hint {
        Some(hint) => format!(
            "Write a children's story with this theme or title idea: \"{}\". Start with a creative title on the first line, then the story.",
            hint
        ),
        None => "Write an original children's story. Start with a creative title on the first line, then the story.".to_string(),
    }
}

/// Build a continuation prompt for generating the next part of a story
pub fn build_continuation_prompt(previous_summary: &str, part_number: i32) -> String {
    format!(
        r#"Continue this children's story with Part {}.

Summary of what happened so far:
{}

Write the next part of the story (300-500 words). Continue the adventure with the same characters. Start directly with the story text - do not include a title or "Part N" heading."#,
        part_number, previous_summary
    )
}

/// Build a summarization prompt for creating a story summary
pub fn build_summary_prompt(story_text: &str) -> String {
    format!(
        "Summarize this children's story in 2-3 sentences, capturing the key characters, setting, and events:\n\n{}",
        story_text
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_contains_required_elements() {
        let prompt = build_system_prompt("Adventure", "Exciting journeys and brave heroes");
        assert!(prompt.contains("children"));
        assert!(prompt.contains("Adventure"));
        assert!(prompt.contains("Exciting journeys"));
    }

    #[test]
    fn story_prompt_with_hint() {
        let prompt = build_story_prompt(Some("A brave cat"));
        assert!(prompt.contains("A brave cat"));
    }

    #[test]
    fn story_prompt_without_hint() {
        let prompt = build_story_prompt(None);
        assert!(prompt.contains("original"));
    }

    #[test]
    fn continuation_prompt_includes_summary() {
        let prompt = build_continuation_prompt("A rabbit went on a journey.", 2);
        assert!(prompt.contains("Part 2"));
        assert!(prompt.contains("A rabbit went on a journey."));
    }

    #[test]
    fn summary_prompt_includes_text() {
        let prompt = build_summary_prompt("Once upon a time...");
        assert!(prompt.contains("Once upon a time..."));
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn system_prompt_always_contains_children(
            genre_name in "[a-zA-Z ]{1,50}",
            genre_desc in "[a-zA-Z ]{1,100}"
        ) {
            let prompt = build_system_prompt(&genre_name, &genre_desc);
            prop_assert!(prompt.contains("children"));
            prop_assert!(prompt.contains(&genre_name));
            prop_assert!(prompt.contains(&genre_desc));
        }

        #[test]
        fn continuation_prompt_always_contains_summary(
            summary in "[a-zA-Z .]{1,200}",
            part_num in 2..100i32
        ) {
            let prompt = build_continuation_prompt(&summary, part_num);
            prop_assert!(prompt.contains(&summary));
            let expected_part = format!("Part {}", part_num);
            prop_assert!(prompt.contains(&expected_part));
        }
    }
}
