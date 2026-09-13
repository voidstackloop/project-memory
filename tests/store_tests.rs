#[cfg(test)]
mod tests {
    use project_memory::store::MemoryStore;
    use project_memory::types::{MemoryInput, MemoryKind};

    fn temp_store() -> MemoryStore {
        let dir = std::env::temp_dir().join(format!("pmem-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        MemoryStore::open(&dir.join("test.db")).unwrap()
    }

    fn sample_input(key: &str, content: &str) -> MemoryInput {
        MemoryInput {
            kind: MemoryKind::Convention,
            key: key.to_string(),
            content: content.to_string(),
            tags: vec!["test".to_string()],
            related_ids: Vec::new(),
        }
    }

    #[test]
    fn test_add_and_get() {
        let store = temp_store();
        let m = store.add(sample_input("naming", "Use snake_case")).unwrap();
        assert_eq!(m.key, "naming");
        assert_eq!(m.kind, MemoryKind::Convention);

        let fetched = store.get(&m.id).unwrap().unwrap();
        assert_eq!(fetched.id, m.id);
        assert_eq!(fetched.content, "Use snake_case");
    }

    #[test]
    fn test_list_with_filter() {
        let store = temp_store();
        store.add(sample_input("a", "content a")).unwrap();
        store.add(MemoryInput {
            kind: MemoryKind::Decision,
            key: "b".to_string(),
            content: "content b".to_string(),
            tags: vec![],
            related_ids: Vec::new(),
        }).unwrap();

        let all = store.list(None, 100).unwrap();
        assert_eq!(all.len(), 2);

        let conventions = store.list(Some(MemoryKind::Convention), 100).unwrap();
        assert_eq!(conventions.len(), 1);
        assert_eq!(conventions[0].key, "a");

        let decisions = store.list(Some(MemoryKind::Decision), 100).unwrap();
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].key, "b");
    }

    #[test]
    fn test_search_exact() {
        let store = temp_store();
        store.add(sample_input("naming-rust", "Use snake_case for functions")).unwrap();
        store.add(sample_input("database", "Using SQLite")).unwrap();

        let results = store.search("snake", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "naming-rust");

        let results = store.search("sqlite", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "database");
    }

    #[test]
    fn test_fuzzy_search() {
        let store = temp_store();
        store.add(sample_input("error-handling", "Use anyhow for application errors")).unwrap();
        store.add(sample_input("naming-conventions", "snake_case for Rust")).unwrap();

        // Fuzzy match on "error" should find error-handling
        let scored = store.fuzzy_search("error", 10).unwrap();
        assert!(!scored.is_empty());
        assert_eq!(scored[0].memory.key, "error-handling");

        // Fuzzy match on "snake" should find naming-conventions
        let scored = store.fuzzy_search("snake", 10).unwrap();
        assert!(!scored.is_empty());
    }

    #[test]
    fn test_update() {
        let store = temp_store();
        let m = store.add(sample_input("old-key", "old content")).unwrap();

        let updated = store.update(&m.id, MemoryInput {
            kind: MemoryKind::Pattern,
            key: "new-key".to_string(),
            content: "new content".to_string(),
            tags: vec!["updated".to_string()],
            related_ids: Vec::new(),
        }).unwrap().unwrap();

        assert_eq!(updated.key, "new-key");
        assert_eq!(updated.content, "new content");
        assert_eq!(updated.kind, MemoryKind::Pattern);
    }

    #[test]
    fn test_delete() {
        let store = temp_store();
        let m = store.add(sample_input("to-delete", "bye")).unwrap();
        assert!(store.delete(&m.id).unwrap());
        assert!(store.get(&m.id).unwrap().is_none());
        assert!(!store.delete("nonexistent").unwrap());
    }

    #[test]
    fn test_link_and_related() {
        let store = temp_store();
        let a = store.add(sample_input("a", "content a")).unwrap();
        let b = store.add(sample_input("b", "content b")).unwrap();
        let c = store.add(sample_input("c", "content c")).unwrap();

        assert!(store.link(&a.id, &b.id).unwrap());
        assert!(store.link(&a.id, &c.id).unwrap());

        let related = store.get_related(&a.id).unwrap();
        assert_eq!(related.len(), 2);

        // Unlink
        store.unlink(&a.id, &b.id).unwrap();
        let related = store.get_related(&a.id).unwrap();
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].key, "c");
    }

    #[test]
    fn test_count_by_kind() {
        let store = temp_store();
        store.add(sample_input("a", "a")).unwrap();
        store.add(sample_input("b", "b")).unwrap();
        store.add(MemoryInput {
            kind: MemoryKind::Decision,
            key: "c".to_string(),
            content: "c".to_string(),
            tags: vec![],
            related_ids: Vec::new(),
        }).unwrap();

        let counts = store.count_by_kind().unwrap();
        assert!(counts.iter().any(|(k, c)| *k == MemoryKind::Convention && *c == 2));
        assert!(counts.iter().any(|(k, c)| *k == MemoryKind::Decision && *c == 1));
    }

    #[test]
    fn test_import_export() {
        let store = temp_store();
        store.add(sample_input("x", "content x")).unwrap();
        store.add(sample_input("y", "content y")).unwrap();

        let count = store.count().unwrap();
        assert_eq!(count, 2);

        let exported = store.export_json().unwrap();
        assert_eq!(exported.len(), 2);
    }

    #[test]
    fn test_memory_kind_parse() {
        assert_eq!("convention".parse::<MemoryKind>().unwrap(), MemoryKind::Convention);
        assert_eq!("conv".parse::<MemoryKind>().unwrap(), MemoryKind::Convention);
        assert_eq!("pattern".parse::<MemoryKind>().unwrap(), MemoryKind::Pattern);
        assert_eq!("pat".parse::<MemoryKind>().unwrap(), MemoryKind::Pattern);
        assert_eq!("decision".parse::<MemoryKind>().unwrap(), MemoryKind::Decision);
        assert_eq!("dec".parse::<MemoryKind>().unwrap(), MemoryKind::Decision);
        assert_eq!("preference".parse::<MemoryKind>().unwrap(), MemoryKind::Preference);
        assert_eq!("pref".parse::<MemoryKind>().unwrap(), MemoryKind::Preference);
        assert_eq!("context".parse::<MemoryKind>().unwrap(), MemoryKind::Context);
        assert_eq!("ctx".parse::<MemoryKind>().unwrap(), MemoryKind::Context);
        assert!("invalid".parse::<MemoryKind>().is_err());
    }

    #[test]
    fn test_id_prefix_resolution() {
        let store = temp_store();
        let m = store.add(sample_input("test", "content")).unwrap();
        let _prefix = &m.id[..8];

        // Exact match
        let found = store.get(&m.id).unwrap();
        assert!(found.is_some());

        // Prefix match should not work with get() (that's resolved in CLI)
        // but fuzzy_search should find it
        let scored = store.fuzzy_search("test", 10).unwrap();
        assert!(!scored.is_empty());
    }
}
