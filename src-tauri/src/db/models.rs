use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub display_order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    pub id: String,
    pub title: String,
    pub genre_id: String,
    pub status: String,
    pub is_sample: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryPart {
    pub id: String,
    pub story_id: String,
    pub part_number: i32,
    pub content: String,
    pub audio_path: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioJob {
    pub id: String,
    pub story_part_id: String,
    pub voice_path: Option<String>,
    pub final_path: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genre_serialization_roundtrip() {
        let genre = Genre {
            id: "adventure".to_string(),
            name: "Adventure".to_string(),
            description: "Exciting journeys".to_string(),
            icon: Some("compass".to_string()),
            display_order: 0,
            created_at: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&genre).unwrap();
        let deserialized: Genre = serde_json::from_str(&json).unwrap();
        assert_eq!(genre.id, deserialized.id);
        assert_eq!(genre.name, deserialized.name);
    }

    #[test]
    fn story_serialization_roundtrip() {
        let story = Story {
            id: "test-id".to_string(),
            title: "Test Story".to_string(),
            genre_id: "adventure".to_string(),
            status: "draft".to_string(),
            is_sample: 0,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&story).unwrap();
        let deserialized: Story = serde_json::from_str(&json).unwrap();
        assert_eq!(story.id, deserialized.id);
        assert_eq!(story.title, deserialized.title);
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn genre_json_roundtrip(
            id in "[a-z]{1,20}",
            name in "[a-zA-Z ]{1,50}",
            desc in "[a-zA-Z .]{1,100}"
        ) {
            let genre = Genre {
                id: id.clone(),
                name: name.clone(),
                description: desc.clone(),
                icon: Some("compass".to_string()),
                display_order: 0,
                created_at: "2024-01-01".to_string(),
            };
            let json = serde_json::to_string(&genre).unwrap();
            let back: Genre = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(genre.id, back.id);
            prop_assert_eq!(genre.name, back.name);
            prop_assert_eq!(genre.description, back.description);
        }

        #[test]
        fn story_json_roundtrip(
            id in "[a-z0-9-]{1,36}",
            title in "[a-zA-Z .!]{1,100}",
            genre_id in "[a-z]{1,20}"
        ) {
            let story = Story {
                id: id.clone(),
                title: title.clone(),
                genre_id: genre_id.clone(),
                status: "draft".to_string(),
                is_sample: 0,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            };
            let json = serde_json::to_string(&story).unwrap();
            let back: Story = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(story.id, back.id);
            prop_assert_eq!(story.title, back.title);
            prop_assert_eq!(story.status, back.status);
        }

        #[test]
        fn all_genre_ids_are_nonempty(
            id in "[a-z]{1,20}"
        ) {
            prop_assert!(!id.is_empty());
        }
    }
}
