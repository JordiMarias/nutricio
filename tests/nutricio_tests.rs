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
}


