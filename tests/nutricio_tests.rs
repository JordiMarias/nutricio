#[cfg(test)]
mod tests {
    use nutricio::models::*;
    use nutricio::scraper::parse_bonpreu_html;
    use nutricio::storage::seed;

    #[test]
    fn test_spreadsheet_day1_calculations() {
        let ingredients = seed::get_seed_ingredients();
        let dishes = seed::get_seed_dishes();
        let weekly = seed::get_seed_weekly_menu();

        let day1 = &weekly.days[0];
        let total = day1.calculate_total_nutrition(&ingredients, &dishes);

        // Day 1 from spreadsheet has 2083.75 Kcal
        assert!((total.kcal - 2083.75).abs() < 10.0, "Expected Kcal around 2083.75, got {}", total.kcal);

        let total_macro = total.total_macro_grams();
        assert!(total_macro > 350.0 && total_macro < 450.0);

        let fat_pct = total.fat_pct();
        let carbs_pct = total.carbs_pct();
        let prot_pct = total.protein_pct();

        // Caloric macro percentages: Fat (9 kcal/g), Carbs (4 kcal/g), Protein (4 kcal/g)
        assert!((fat_pct - 36.27).abs() < 2.0, "Expected Fat Kcal % around 36.27%, got {:.2}", fat_pct);
        assert!((carbs_pct - 34.33).abs() < 3.0, "Expected Carbs Kcal % around 34.33%, got {:.2}", carbs_pct);
        assert!((prot_pct - 29.40).abs() < 3.0, "Expected Protein Kcal % around 29.40%, got {:.2}", prot_pct);
    }

    #[test]
    fn test_nova_classifier() {
        assert_eq!(detect_nova_group("Poma fresca", None), NovaGroup::Group1Unprocessed);
        assert_eq!(detect_nova_group("Oli d'oliva", None), NovaGroup::Group2ProcessedIngredient);
        assert_eq!(detect_nova_group("Galetes industrials", Some("Farina, sucre, oli de palma, aroma, e-471")), NovaGroup::Group4UltraProcessed);
    }

    #[test]
    fn test_bonpreu_html_parsing() {
        let mock_html = r#"
            <script type="application/ld+json">
            {
              "@context": "https://schema.org",
              "@type": "Product",
              "name": "BONPREU Arròs basmati integral ecològic",
              "brand": "BONPREU",
              "size": "0.5kg",
              "offers": { "price": "2.15", "priceCurrency": "EUR" }
            }
            </script>
            <h2>Dades nutricionals</h2>
            <table>
              <tr><td>Valors energètics</td><td>1519 kJ / 359 kcal</td></tr>
              <tr><td>Greixos</td><td>2,6 g</td></tr>
              <tr><td>dels quals saturats</td><td>0,6 g</td></tr>
              <tr><td>Hidrats de carboni</td><td>74 g</td></tr>
              <tr><td>dels quals sucres</td><td>1,0 g</td></tr>
              <tr><td>Proteïnes</td><td>8,3 g</td></tr>
              <tr><td>Sal</td><td>0g</td></tr>
            </table>
            <h2>Ingredients</h2>
            <p>INGREDIENTS: arròs basmati espellofat de gra llarg*.</p>
        "#;

        let res = parse_bonpreu_html(mock_html, "https://example.com").unwrap();
        assert_eq!(res.name, "BONPREU Arròs basmati integral ecològic");
        assert_eq!(res.price, Some(2.15));
        assert_eq!(res.nutrition_100g.kcal, 359.0);
        assert_eq!(res.nutrition_100g.fat_g, 2.6);
        assert_eq!(res.nutrition_100g.protein_g, 8.3);
        assert_eq!(res.nova_group, NovaGroup::Group1Unprocessed);
    }

    #[test]
    fn test_llenties_pardina_scraping() {
        let mock_html = r#"
            <script type="application/ld+json">
            {"@context":"https://schema.org","@type":"Product","sku":"11263","name":"ECOBASICS Llenties pardina ecològiques","description":"Llenties pardina ecològiques Ecobasics en bossa de 500 grams<br>","brand":"ECOBASICS","size":"0.5kg","offers":{"@type":"Offer","price":"2.99","priceCurrency":"EUR"}}
            </script>
            <h2>Dades nutricionals</h2>
            <table><tbody>
            <tr><td>Valor energètic </td><td> 916 kJ / 366 kcal </td></tr>
            <tr><td>Greixos </td><td> 2,5 g </td></tr>
            <tr><td>dels quals saturats </td><td> 0,9 g </td></tr>
            <tr><td>Hidrats de carboni </td><td> 56 g</td></tr>
            <tr><td>dels quals sucres </td><td> 4,3 g</td></tr>
            <tr><td>Fibra alimentària </td><td> 17 g</td></tr>
            <tr><td>Proteïnes </td><td> 25 g</td></tr>
            <tr><td>Sal </td><td> 0,01 g</td></tr>
            </tbody></table>
        "#;

        let res = parse_bonpreu_html(mock_html, "https://compraonline.bonpreuesclat.cat/products/ecobasics-llenties-pardina-ecol%C3%B2giques/11263").unwrap();
        assert_eq!(res.name, "ECOBASICS Llenties pardina ecològiques");
        assert_eq!(res.price, Some(2.99));
        assert_eq!(res.nutrition_100g.kcal, 366.0);
        assert_eq!(res.nutrition_100g.fat_g, 2.5);
        assert_eq!(res.nutrition_100g.saturated_fat_g, 0.9);
        assert_eq!(res.nutrition_100g.carbs_g, 56.0);
        assert_eq!(res.nutrition_100g.sugars_g, 4.3);
        assert_eq!(res.nutrition_100g.fiber_g, 17.0);
        assert_eq!(res.nutrition_100g.protein_g, 25.0);
        assert_eq!(res.nutrition_100g.salt_g, 0.01);
    }

    #[test]
    fn test_liquid_parsing() {
        let mock_html = r#"
            <script type="application/ld+json">
            {"@context":"https://schema.org","@type":"Product","sku":"99999","name":"BONPREU Llet sencer ecològica 1.5L","description":"Llet de vaca en ampolla de 1.5L","offers":{"@type":"Offer","price":"1.80","priceCurrency":"EUR"}}
            </script>
            <h2>Dades nutricionals</h2>
            <table><tbody>
            <tr><td>Valor energètic </td><td> 260 kJ / 62 kcal </td></tr>
            <tr><td>Greixos </td><td> 3,6 g </td></tr>
            <tr><td>Proteïnes </td><td> 3,2 g </td></tr>
            </tbody></table>
        "#;

        let res = parse_bonpreu_html(mock_html, "https://example.com/llet").unwrap();
        assert_eq!(res.name, "BONPREU Llet sencer ecològica 1.5L");
        assert_eq!(res.price, Some(1.80));
        assert_eq!(res.pack_weight_g, Some(1500.0));
        // Marginal price per 100ml: (1.80 / 1500) * 100 = 0.12 EUR
        assert!((res.nutrition_100g.price_euro - 0.12).abs() < 0.001);
    }

    #[test]
    fn test_glycemic_index_and_load() {
        let ing_lentils = Ingredient::new(
            "ing_llenties",
            "Llenties pardines ecològiques",
            NutritionalInfo {
                kcal: 320.0,
                fat_g: 1.5,
                saturated_fat_g: 0.2,
                carbs_g: 48.0,
                sugars_g: 1.2,
                fiber_g: 18.0,
                protein_g: 24.0,
                salt_g: 0.01,
                price_euro: 0.25,
            },
        );

        let ig = ing_lentils.get_glycemic_index();
        assert!(ig <= 55, "Lentils should be estimated as low GI");

        // Portion of 100g: 48g carbs -> CG = (35 * 48) / 100 = 16.8
        let cg = ing_lentils.calculate_glycemic_load(100.0);
        assert!((cg - 16.8).abs() < 1.0);
    }

    #[test]
    fn test_shopping_list_html_generation() {
        let state = nutricio::storage::AppState::with_seed_data();
        let html = nutricio::exporter::generate_shopping_list_report_html(&state);
        assert!(html.contains("🛒 Llista de la Compra Setmanal"));
        assert!(html.contains("window.print()"));
        assert!(html.contains("Cost Estimat Total Setmanal"));
    }

    #[test]
    fn test_dish_nova_breakdown_per_ingredient() {
        let mut ing1 = Ingredient::new("ing1", "Arròs Integral", NutritionalInfo { kcal: 350.0, ..NutritionalInfo::zero() });
        ing1.nova_group = Some(NovaGroup::Group1Unprocessed);

        let mut ing4 = Ingredient::new("ing4", "Salsa Industrial", NutritionalInfo { kcal: 50.0, ..NutritionalInfo::zero() });
        ing4.nova_group = Some(NovaGroup::Group4UltraProcessed);

        let ingredients = vec![ing1, ing4];

        let dish = Dish {
            id: "dish_mixed".to_string(),
            name: "Arròs amb Salsa".to_string(),
            description: None,
            items: vec![
                DishItem { ingredient_id: "ing1".to_string(), quantity: 100.0 }, // 350 Kcal (NOVA 1)
                DishItem { ingredient_id: "ing4".to_string(), quantity: 100.0 }, // 50 Kcal (NOVA 4)
            ],
            servings: 1.0,
        };

        // Predominant group for the dish itself should be Group 1 (350 kcal vs 50 kcal)
        assert_eq!(dish.derived_nova_group(&ingredients), NovaGroup::Group1Unprocessed);

        let dishes = vec![dish];

        let mut day = DailyMenu::new("Dilluns");
        let entries = day.meals.get_mut(&MealType::Lunch).unwrap();
        entries.push(MealEntry {
            id: "entry1".to_string(),
            item_id: "dish_mixed".to_string(),
            is_dish: true,
            quantity: 1.0,
        });

        let breakdown = day.nova_breakdown(&ingredients, &dishes);

        let g1_kcal = breakdown.get(&NovaGroup::Group1Unprocessed).copied().unwrap_or(0.0);
        let g4_kcal = breakdown.get(&NovaGroup::Group4UltraProcessed).copied().unwrap_or(0.0);

        assert!((g1_kcal - 350.0).abs() < 1.0, "Expected 350 Kcal in Group 1, got {}", g1_kcal);
        assert!((g4_kcal - 50.0).abs() < 1.0, "Expected 50 Kcal in Group 4, got {}", g4_kcal);
    }

    #[test]
    fn test_dish_nova_breakdown_percentages() {
        let mut ing1 = Ingredient::new("ing1", "Patata", NutritionalInfo { kcal: 80.0, ..NutritionalInfo::zero() });
        ing1.nova_group = Some(NovaGroup::Group1Unprocessed);

        let mut ing2 = Ingredient::new("ing2", "Oli d'oliva", NutritionalInfo { kcal: 900.0, ..NutritionalInfo::zero() });
        ing2.nova_group = Some(NovaGroup::Group2ProcessedIngredient);

        let mut ing4 = Ingredient::new("ing4", "Ketchup Industrial", NutritionalInfo { kcal: 160.0, ..NutritionalInfo::zero() });
        ing4.nova_group = Some(NovaGroup::Group4UltraProcessed);

        let ingredients = vec![ing1, ing2, ing4];

        let dish = Dish {
            id: "dish_patates".to_string(),
            name: "Patates Fregides amb Salsa".to_string(),
            description: None,
            items: vec![
                DishItem { ingredient_id: "ing1".to_string(), quantity: 100.0 }, // 80 Kcal
                DishItem { ingredient_id: "ing2".to_string(), quantity: 20.0 },  // 180 Kcal
                DishItem { ingredient_id: "ing4".to_string(), quantity: 25.0 },  // 40 Kcal
            ],
            servings: 1.0,
        };

        let breakdown = dish.nova_breakdown(&ingredients);
        let total_nut = dish.calculate_total_nutrition(&ingredients);
        assert!((total_nut.kcal - 300.0).abs() < 1.0);

        let g1 = breakdown.get(&NovaGroup::Group1Unprocessed).copied().unwrap_or(0.0);
        let g2 = breakdown.get(&NovaGroup::Group2ProcessedIngredient).copied().unwrap_or(0.0);
        let g3 = breakdown.get(&NovaGroup::Group3Processed).copied().unwrap_or(0.0);
        let g4 = breakdown.get(&NovaGroup::Group4UltraProcessed).copied().unwrap_or(0.0);

        assert!((g1 - 80.0).abs() < 1.0);
        assert!((g2 - 180.0).abs() < 1.0);
        assert_eq!(g3, 0.0);
        assert!((g4 - 40.0).abs() < 1.0);

        // Check percentage calculation
        let g1_pct = (g1 / total_nut.kcal) * 100.0;
        let g2_pct = (g2 / total_nut.kcal) * 100.0;
        let g4_pct = (g4 / total_nut.kcal) * 100.0;

        assert!((g1_pct - 26.67).abs() < 0.5);
        assert!((g2_pct - 60.0).abs() < 0.5);
        assert!((g4_pct - 13.33).abs() < 0.5);
    }

    #[test]
    fn test_daily_journal_tracking_and_serialization() {
        let mut state = nutricio::storage::AppState::empty();

        let mut apple = Ingredient::new("ing_poma", "Poma Golden", NutritionalInfo {
            kcal: 52.0,
            fat_g: 0.2,
            saturated_fat_g: 0.0,
            carbs_g: 14.0,
            sugars_g: 10.0,
            fiber_g: 2.4,
            protein_g: 0.3,
            salt_g: 0.0,
            price_euro: 0.40,
        });
        apple.nova_group = Some(NovaGroup::Group1Unprocessed);
        state.ingredients.push(apple);

        // Add to breakfast in daily journal
        {
            let log = state.get_or_create_daily_log("2026-09-08", "Dimarts");
            let entries = log.daily_menu.meals.entry(MealType::Breakfast).or_insert_with(Vec::new);
            entries.push(MealEntry {
                id: "entry_1".to_string(),
                item_id: "ing_poma".to_string(),
                is_dish: false,
                quantity: 150.0, // 150g -> 78 kcal
            });
        }

        let total = state.daily_journal[0].daily_menu.calculate_total_nutrition(&state.ingredients, &state.dishes);
        assert!((total.kcal - 78.0).abs() < 0.1);

        // Test JSON serialization & roundtrip
        let json = nutricio::storage::save_state_to_json(&state).expect("Serialization failed");
        let loaded = nutricio::storage::load_state_from_json(&json).expect("Deserialization failed");

        assert_eq!(loaded.daily_journal.len(), 1);
        assert_eq!(loaded.daily_journal[0].date, "2026-09-08");
        assert_eq!(loaded.daily_journal[0].daily_menu.meals.get(&MealType::Breakfast).unwrap().len(), 1);
    }

    #[test]
    fn test_merge_database_from_json() {
        let mut state = nutricio::storage::AppState::empty();

        let json_data = r#"{
            "ingredients": [
                {
                    "id": "bonpreu_1",
                    "name": "BONPREU Farina integral de blat",
                    "brand": "BONPREU",
                    "source_url": null,
                    "unit_type": "Per100g",
                    "per_unit_nutrition": {
                        "kcal": 326.0,
                        "fat_g": 2.2,
                        "saturated_fat_g": 0.7,
                        "carbs_g": 60.0,
                        "sugars_g": 2.0,
                        "fiber_g": 11.0,
                        "protein_g": 11.0,
                        "salt_g": 0.0,
                        "price_euro": 0.089
                    },
                    "price_per_pack": 0.89,
                    "pack_weight_g": 1000.0,
                    "nova_group": "Group1Unprocessed",
                    "glycemic_index": 35,
                    "ingredients_text": "Farina integral"
                }
            ],
            "dishes": [
                {
                    "id": "dish_1",
                    "name": "Pizza Integral",
                    "description": null,
                    "items": [
                        { "ingredient_id": "bonpreu_1", "quantity": 100.0 }
                    ],
                    "servings": 1.0
                }
            ]
        }"#;

        let res = state.merge_database_from_json(json_data);
        assert!(res.is_ok());
        let (n_ing, n_dish) = res.unwrap();
        assert_eq!(n_ing, 1);
        assert_eq!(n_dish, 1);
        assert_eq!(state.ingredients.len(), 1);
        assert_eq!(state.ingredients[0].name, "BONPREU Farina integral de blat");
        assert_eq!(state.dishes.len(), 1);

        // Test updating an existing ingredient
        let updated_json = r#"{
            "aliments": [
                {
                    "id": "bonpreu_1",
                    "name": "BONPREU Farina integral de blat",
                    "brand": "BONPREU",
                    "source_url": null,
                    "unit_type": "Per100g",
                    "per_unit_nutrition": {
                        "kcal": 330.0,
                        "fat_g": 2.5,
                        "saturated_fat_g": 0.7,
                        "carbs_g": 60.0,
                        "sugars_g": 2.0,
                        "fiber_g": 11.0,
                        "protein_g": 12.0,
                        "salt_g": 0.0,
                        "price_euro": 0.095
                    },
                    "price_per_pack": 0.95,
                    "pack_weight_g": 1000.0,
                    "nova_group": "Group1Unprocessed",
                    "glycemic_index": 35,
                    "ingredients_text": "Farina integral 100%"
                }
            ]
        }"#;

        let res2 = state.merge_database_from_json(updated_json);
        assert!(res2.is_ok());
        assert_eq!(state.ingredients.len(), 1); // Still 1 ingredient, updated
        assert_eq!(state.ingredients[0].per_unit_nutrition.kcal, 330.0);
        assert_eq!(state.ingredients[0].price_per_pack, Some(0.95));
    }

    #[test]
    fn test_load_base_database_json_file() {
        let mut state = nutricio::storage::AppState::empty();
        let file_content = std::fs::read_to_string("base_database.json").expect("base_database.json should exist");
        let res = state.merge_database_from_json(&file_content);
        assert!(res.is_ok(), "Failed to parse base_database.json: {:?}", res.err());
        let (n_ing, n_dish) = res.unwrap();
        assert_eq!(n_ing, 100);
        assert_eq!(n_dish, 1);
        assert_eq!(state.ingredients.len(), 100);
        assert_eq!(state.dishes.len(), 1);

        // Verify that all IDs are unique and start with base_
        let mut seen_ids = std::collections::HashSet::new();
        for ing in &state.ingredients {
            assert!(ing.id.starts_with("base_"), "Ingredient id {} should start with base_", ing.id);
            assert!(seen_ids.insert(ing.id.clone()), "Duplicate id: {}", ing.id);
        }

        // Verify dish 1 ingredients exist in database
        let dish = &state.dishes[0];
        assert_eq!(dish.id, "base_dish_1");
        for item in &dish.items {
            assert!(seen_ids.contains(&item.ingredient_id), "Dish ingredient {} not in DB", item.ingredient_id);
        }

        // Test dynamic ID generator guarantees no collision
        let new_manual_id = state.generate_unique_ingredient_id("manual");
        assert_eq!(new_manual_id, "manual_1");

        let new_base_id = state.generate_unique_ingredient_id("base");
        assert_eq!(new_base_id, "base_101");
    }

    #[test]
    fn test_parse_punctual_ingredient_json() {
        use nutricio::views::parse_punctual_ingredient_json;

        // 1. Direct standard base_database format
        let standard_json = r#"{
            "id": "ai_est_1",
            "name": "Paella marinera de restaurant",
            "brand": "Restaurant El Port",
            "source_url": null,
            "unit_type": "Per100g",
            "per_unit_nutrition": {
                "kcal": 165.0,
                "fat_g": 5.2,
                "saturated_fat_g": 1.1,
                "carbs_g": 22.0,
                "sugars_g": 1.0,
                "fiber_g": 1.5,
                "protein_g": 7.5,
                "salt_g": 1.2,
                "price_euro": 0.0
            },
            "price_per_pack": 18.5,
            "pack_weight_g": 350.0,
            "nova_group": "Group3Processed",
            "glycemic_index": 60,
            "ingredients_text": "Arròs, sèpia, gambes, sofregit de ceba i tomàquet, brou de peix, safrà"
        }"#;

        let parsed = parse_punctual_ingredient_json(standard_json).expect("Standard format should parse");
        assert_eq!(parsed.name, "Paella marinera de restaurant");
        assert_eq!(parsed.brand.as_deref(), Some("Restaurant El Port"));
        assert_eq!(parsed.per_unit_nutrition.kcal, 165.0);
        assert_eq!(parsed.per_unit_nutrition.protein_g, 7.5);
        assert_eq!(parsed.nova_group, Some(NovaGroup::Group3Processed));
        assert_eq!(parsed.pack_weight_g, Some(350.0));

        // 2. Wrapped format {"ingredients": [...]}
        let wrapped_json = r#"{
            "ingredients": [
                {
                    "name": "Tiramisú artesanal",
                    "per_unit_nutrition": {
                        "kcal": 290.0,
                        "fat_g": 16.0,
                        "saturated_fat_g": 9.0,
                        "carbs_g": 30.0,
                        "sugars_g": 22.0,
                        "fiber_g": 1.0,
                        "protein_g": 5.0,
                        "salt_g": 0.2,
                        "price_euro": 0.0
                    },
                    "nova_group": "Group4UltraProcessed"
                }
            ]
        }"#;

        let parsed_wrapped = parse_punctual_ingredient_json(wrapped_json).expect("Wrapped format should parse");
        assert_eq!(parsed_wrapped.name, "Tiramisú artesanal");
        assert_eq!(parsed_wrapped.per_unit_nutrition.kcal, 290.0);
        assert_eq!(parsed_wrapped.nova_group, Some(NovaGroup::Group4UltraProcessed));

        // 3. Lenient AI prompt output format (flat keys, numeric nova)
        let lenient_json = r#"{
            "name": "Entrecot a la brasa amb patates",
            "calories": 240,
            "protein": 22,
            "carbs": 12,
            "fat": 11,
            "fiber": 1.2,
            "sugar": 0.5,
            "nova": 3,
            "serving_size_g": 400
        }"#;

        let parsed_lenient = parse_punctual_ingredient_json(lenient_json).expect("Lenient AI format should parse");
        assert_eq!(parsed_lenient.name, "Entrecot a la brasa amb patates");
        assert_eq!(parsed_lenient.per_unit_nutrition.kcal, 240.0);
        assert_eq!(parsed_lenient.per_unit_nutrition.protein_g, 22.0);
        assert_eq!(parsed_lenient.per_unit_nutrition.carbs_g, 12.0);
        assert_eq!(parsed_lenient.per_unit_nutrition.fat_g, 11.0);
        assert_eq!(parsed_lenient.nova_group, Some(NovaGroup::Group3Processed));
        assert_eq!(parsed_lenient.pack_weight_g, Some(400.0));
    }

    #[test]
    fn test_punctual_ingredients_storage_and_daily_journal() {
        use nutricio::storage::{AppState, load_state_from_json, save_state_to_json};

        let mut state = AppState::empty();

        // 1. Regular database ingredient
        let regular_food = Ingredient {
            id: "base_1".to_string(),
            name: "Arròs".to_string(),
            brand: None,
            source_url: None,
            unit_type: UnitType::Per100g,
            per_unit_nutrition: NutritionalInfo {
                kcal: 350.0,
                fat_g: 1.0,
                saturated_fat_g: 0.2,
                carbs_g: 78.0,
                sugars_g: 0.5,
                fiber_g: 1.0,
                protein_g: 7.0,
                salt_g: 0.01,
                price_euro: 0.2,
            },
            price_per_pack: None,
            pack_weight_g: None,
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: Some(70),
            ingredients_text: None,
        };
        state.ingredients.push(regular_food);

        // 2. Add punctual restaurant food into separate punctual_ingredients list
        let punctual_id = state.generate_unique_punctual_id();
        assert_eq!(punctual_id, "punctual_1");

        let punctual_food = Ingredient {
            id: punctual_id.clone(),
            name: "Plat combinat restaurant (estimat)".to_string(),
            brand: Some("Gourmet Bar".to_string()),
            source_url: None,
            unit_type: UnitType::Per100g,
            per_unit_nutrition: NutritionalInfo {
                kcal: 200.0,
                fat_g: 10.0,
                saturated_fat_g: 3.0,
                carbs_g: 15.0,
                sugars_g: 2.0,
                fiber_g: 2.0,
                protein_g: 12.0,
                salt_g: 1.5,
                price_euro: 0.0,
            },
            price_per_pack: Some(15.0),
            pack_weight_g: Some(300.0),
            nova_group: Some(NovaGroup::Group3Processed),
            glycemic_index: Some(55),
            ingredients_text: Some("Pollastre rostit, patates al forn, amanida".to_string()),
        };
        state.punctual_ingredients.push(punctual_food);

        // Verify isolation: ingredients has 1, punctual_ingredients has 1
        assert_eq!(state.ingredients.len(), 1);
        assert_eq!(state.punctual_ingredients.len(), 1);

        // Verify all_ingredients combines both
        let all = state.all_ingredients();
        assert_eq!(all.len(), 2);
        assert_eq!(state.find_ingredient("base_1").unwrap().name, "Arròs");
        assert_eq!(state.find_ingredient("punctual_1").unwrap().name, "Plat combinat restaurant (estimat)");

        // 3. Add meal entry referencing punctual food in DailyLog
        {
            let daily_log = state.get_or_create_daily_log("2026-10-01", "Dijous");
            let lunch = daily_log.daily_menu.meals.entry(MealType::Lunch).or_default();
            lunch.push(MealEntry {
                id: "entry_1".to_string(),
                item_id: "punctual_1".to_string(),
                is_dish: false,
                quantity: 300.0, // 300g at 200 kcal/100g = 600 kcal
            });
        }

        // Compute nutrition using all_ingredients
        let all_ings = state.all_ingredients();
        let daily_log = state.daily_journal.iter().find(|l| l.date == "2026-10-01").unwrap();
        let total = daily_log.daily_menu.calculate_total_nutrition(&all_ings, &state.dishes);
        assert!((total.kcal - 600.0).abs() < 1e-4, "Expected 600 kcal, got {}", total.kcal);
        assert!((total.protein_g - 36.0).abs() < 1e-4, "Expected 36g protein, got {}", total.protein_g);
        assert!((total.fat_g - 30.0).abs() < 1e-4, "Expected 30g fat, got {}", total.fat_g);
        assert!((total.carbs_g - 45.0).abs() < 1e-4, "Expected 45g carbs, got {}", total.carbs_g);

        // 4. Test serialization and deserialization
        let json_str = save_state_to_json(&state).expect("State should serialize to JSON");
        assert!(json_str.contains("punctual_ingredients"));
        assert!(json_str.contains("punctual_1"));

        let loaded: AppState = load_state_from_json(&json_str).expect("State should deserialize from JSON");
        assert_eq!(loaded.ingredients.len(), 1);
        assert_eq!(loaded.punctual_ingredients.len(), 1);
        assert_eq!(loaded.punctual_ingredients[0].id, "punctual_1");
        assert_eq!(loaded.punctual_ingredients[0].name, "Plat combinat restaurant (estimat)");
        assert_eq!(loaded.daily_journal.len(), 1);

        // 5. Test alias support: JSON with "punctual" key instead of "punctual_ingredients"
        let alias_json = json_str.replace("\"punctual_ingredients\":", "\"punctual\":");
        let loaded_alias: AppState = load_state_from_json(&alias_json).expect("State with alias 'punctual' should deserialize");
        assert_eq!(loaded_alias.punctual_ingredients.len(), 1);
        assert_eq!(loaded_alias.punctual_ingredients[0].id, "punctual_1");
    }
}



