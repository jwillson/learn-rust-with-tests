use std::collections::HashMap;
use std::sync::Mutex;

// ANCHOR: trait
pub trait RecipeStore {
    fn add(&self, recipe: &str);
    fn all(&self) -> Vec<String>;
}
// ANCHOR_END: trait

// ANCHOR: fake
#[derive(Default)]
pub struct InMemoryRecipeStore {
    recipes: Mutex<Vec<String>>,
}

impl RecipeStore for InMemoryRecipeStore {
    fn add(&self, recipe: &str) {
        self.recipes.lock().unwrap().push(recipe.to_string());
    }

    fn all(&self) -> Vec<String> {
        self.recipes.lock().unwrap().clone()
    }
}
// ANCHOR_END: fake

// ANCHOR: json_store
/// A "real" store that persists to a JSON string it owns — standing in for a
/// database-backed implementation. It behaves differently on the inside but
/// must satisfy the same contract.
#[derive(Default)]
pub struct JsonRecipeStore {
    document: Mutex<String>,
}

impl RecipeStore for JsonRecipeStore {
    fn add(&self, recipe: &str) {
        let mut document = self.document.lock().unwrap();
        let mut recipes: Vec<String> = if document.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&document).unwrap_or_default()
        };
        recipes.push(recipe.to_string());
        *document = serde_json::to_string(&recipes).unwrap();
    }

    fn all(&self) -> Vec<String> {
        let document = self.document.lock().unwrap();
        if document.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&document).unwrap_or_default()
        }
    }
}
// ANCHOR_END: json_store

// ANCHOR: contract
/// The contract: a reusable set of assertions any `RecipeStore` must satisfy.
/// It tests *state*, not interactions — add recipes, then read them back.
pub fn recipe_store_contract(store: &impl RecipeStore) {
    assert!(store.all().is_empty(), "a fresh store starts empty");

    store.add("Ramen");
    store.add("Pancakes");

    let got = store.all();
    assert!(got.contains(&"Ramen".to_string()), "{got:?}");
    assert!(got.contains(&"Pancakes".to_string()), "{got:?}");
    assert_eq!(got.len(), 2, "{got:?}");
}
// ANCHOR_END: contract

// ANCHOR: consumer
/// A little domain service that depends on a store. In tests we hand it the
/// in-memory fake and assert on the store's final state — no spying.
pub struct MenuPlanner<'a> {
    store: &'a dyn RecipeStore,
}

impl<'a> MenuPlanner<'a> {
    pub fn new(store: &'a dyn RecipeStore) -> MenuPlanner<'a> {
        MenuPlanner { store }
    }

    pub fn plan(&self, recipes: &[&str]) {
        for recipe in recipes {
            self.store.add(recipe);
        }
    }

    pub fn count(&self) -> usize {
        self.store.all().len()
    }
}

pub fn recipe_counts(store: &impl RecipeStore) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for recipe in store.all() {
        *counts.entry(recipe).or_insert(0) += 1;
    }
    counts
}
// ANCHOR_END: consumer

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: contract_tests
    #[test]
    fn in_memory_store_satisfies_the_contract() {
        recipe_store_contract(&InMemoryRecipeStore::default());
    }

    #[test]
    fn json_store_satisfies_the_contract() {
        recipe_store_contract(&JsonRecipeStore::default());
    }
    // ANCHOR_END: contract_tests

    // ANCHOR: state_test
    #[test]
    fn planning_a_menu_records_every_recipe() {
        let store = InMemoryRecipeStore::default();
        let planner = MenuPlanner::new(&store);

        planner.plan(&["Ramen", "Pancakes", "Ramen"]);

        // Assert on the store's *state*, the way you'd query a real database.
        assert_eq!(planner.count(), 3);
        assert_eq!(recipe_counts(&store)["Ramen"], 2);
    }
    // ANCHOR_END: state_test
}
