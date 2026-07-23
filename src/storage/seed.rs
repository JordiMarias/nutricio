use crate::models::*;

pub fn get_seed_ingredients() -> Vec<Ingredient> {
    let mut list = vec![
        Ingredient {
            id: "cafe_amb_llet".into(),
            name: "Cafè amb llet".into(),
            brand: None,
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 150.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 95.25,
                fat_g: 2.44,
                saturated_fat_g: 0.57,
                carbs_g: 15.38,
                sugars_g: 0.025,
                fiber_g: 2.53,
                protein_g: 1.365,
                salt_g: 0.2105,
                price_euro: 0.35,
            },
            price_per_pack: Some(0.35),
            pack_weight_g: Some(150.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Cafè, llet".into()),
        },
        Ingredient {
            id: "poma".into(),
            name: "Poma".into(),
            brand: None,
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 150.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 52.0,
                fat_g: 0.2,
                saturated_fat_g: 0.0,
                carbs_g: 14.0,
                sugars_g: 10.0,
                fiber_g: 2.4,
                protein_g: 0.3,
                salt_g: 0.0,
                price_euro: 0.30,
            },
            price_per_pack: Some(2.0),
            pack_weight_g: Some(1000.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Poma fresca".into()),
        },
        Ingredient {
            id: "platan".into(),
            name: "Plàtan".into(),
            brand: None,
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 120.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 89.0,
                fat_g: 0.33,
                saturated_fat_g: 0.0,
                carbs_g: 22.84,
                sugars_g: 12.23,
                fiber_g: 2.6,
                protein_g: 1.09,
                salt_g: 0.0,
                price_euro: 0.25,
            },
            price_per_pack: Some(1.80),
            pack_weight_g: Some(1000.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Plàtan fresc".into()),
        },
        Ingredient {
            id: "taronja".into(),
            name: "Taronja".into(),
            brand: None,
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 150.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 47.0,
                fat_g: 0.1,
                saturated_fat_g: 0.0,
                carbs_g: 12.0,
                sugars_g: 9.0,
                fiber_g: 2.4,
                protein_g: 0.9,
                salt_g: 0.0,
                price_euro: 0.25,
            },
            price_per_pack: Some(1.99),
            pack_weight_g: Some(1500.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Taronja fresca".into()),
        },
        Ingredient {
            id: "pera".into(),
            name: "Pera".into(),
            brand: None,
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 140.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 57.0,
                fat_g: 0.1,
                saturated_fat_g: 0.0,
                carbs_g: 15.0,
                sugars_g: 10.0,
                fiber_g: 3.1,
                protein_g: 0.4,
                salt_g: 0.0,
                price_euro: 0.35,
            },
            price_per_pack: Some(2.20),
            pack_weight_g: Some(1000.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Pera fresca".into()),
        },
        Ingredient {
            id: "zespri_kiwi_verd".into(),
            name: "ZESPRI Kiwi verd".into(),
            brand: Some("ZESPRI".into()),
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 100.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 60.0,
                fat_g: 0.5,
                saturated_fat_g: 0.0,
                carbs_g: 15.0,
                sugars_g: 9.0,
                fiber_g: 3.0,
                protein_g: 1.1,
                salt_g: 0.0,
                price_euro: 0.50,
            },
            price_per_pack: Some(2.99),
            pack_weight_g: Some(600.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Kiwi verd fresc".into()),
        },
        Ingredient {
            id: "alvocat".into(),
            name: "Alvocat".into(),
            brand: None,
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 150.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 240.0,
                fat_g: 22.0,
                saturated_fat_g: 3.0,
                carbs_g: 13.0,
                sugars_g: 0.0,
                fiber_g: 10.0,
                protein_g: 3.0,
                salt_g: 0.011,
                price_euro: 0.90,
            },
            price_per_pack: Some(2.50),
            pack_weight_g: Some(500.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Alvocat fresc".into()),
        },
        Ingredient {
            id: "bonpreu_filet_indiot".into(),
            name: "BONPREU Filet d'indiot".into(),
            brand: Some("BONPREU".into()),
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 100.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 107.0,
                fat_g: 1.7,
                saturated_fat_g: 0.6,
                carbs_g: 0.0,
                sugars_g: 0.0,
                fiber_g: 0.0,
                protein_g: 23.0,
                salt_g: 0.19,
                price_euro: 1.10,
            },
            price_per_pack: Some(3.85),
            pack_weight_g: Some(350.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Filet d'indiot".into()),
        },
        Ingredient {
            id: "bonpreu_pit_pollastre".into(),
            name: "BONPREU DE L'ERA Pit filetejat de pollastre".into(),
            brand: Some("BONPREU DE L'ERA".into()),
            source_url: Some("https://www.compraonline.bonpreuesclat.cat/products/bonpreu-pit-filetejat-pollastre/76013".into()),
            unit_type: UnitType::PerUnit { grams_per_unit: 100.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 111.0,
                fat_g: 2.6,
                saturated_fat_g: 0.6,
                carbs_g: 0.0,
                sugars_g: 0.0,
                fiber_g: 0.0,
                protein_g: 22.0,
                salt_g: 0.10,
                price_euro: 1.12,
            },
            price_per_pack: Some(4.49),
            pack_weight_g: Some(400.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Pit de pollastre filetejat".into()),
        },
        Ingredient {
            id: "bonpreu_arros_basmati_bio".into(),
            name: "BONPREU Arròs basmati integral ecològic".into(),
            brand: Some("BONPREU".into()),
            source_url: Some("https://www.compraonline.bonpreuesclat.cat/products/bonpreu-arr%C3%B2s-basmati-integral-ecol%C3%B2gic/85330".into()),
            unit_type: UnitType::Per100g,
            per_unit_nutrition: NutritionalInfo {
                kcal: 359.0,
                fat_g: 2.6,
                saturated_fat_g: 0.6,
                carbs_g: 74.0,
                sugars_g: 1.0,
                fiber_g: 4.0,
                protein_g: 8.3,
                salt_g: 0.0,
                price_euro: 0.43,
            },
            price_per_pack: Some(2.15),
            pack_weight_g: Some(500.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("arròs basmati espellofat de gra llarg*".into()),
        },
        Ingredient {
            id: "llobarro".into(),
            name: "Llobarro (ministeri)".into(),
            brand: None,
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 100.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 84.0,
                fat_g: 1.3,
                saturated_fat_g: 0.27,
                carbs_g: 0.0,
                sugars_g: 0.0,
                fiber_g: 0.0,
                protein_g: 18.0,
                salt_g: 0.10,
                price_euro: 1.20,
            },
            price_per_pack: Some(2.40),
            pack_weight_g: Some(200.0),
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Llobarro de piscicultura".into()),
        },
        Ingredient {
            id: "ing_arros_cigrons".into(),
            name: "Base Arròs amb cigrons".into(),
            brand: None,
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 250.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 667.8,
                fat_g: 13.37,
                saturated_fat_g: 2.15,
                carbs_g: 99.0,
                sugars_g: 3.87,
                fiber_g: 30.51,
                protein_g: 20.34,
                salt_g: 0.0,
                price_euro: 1.25,
            },
            price_per_pack: None,
            pack_weight_g: None,
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Arròs, cigrons cuits, verdures".into()),
        },
        Ingredient {
            id: "ing_amanida_propia".into(),
            name: "Base Amanida pròpia".into(),
            brand: None,
            source_url: None,
            unit_type: UnitType::PerUnit { grams_per_unit: 200.0 },
            per_unit_nutrition: NutritionalInfo {
                kcal: 330.2,
                fat_g: 27.034,
                saturated_fat_g: 3.4405,
                carbs_g: 14.944,
                sugars_g: 6.68,
                fiber_g: 6.54,
                protein_g: 7.359,
                salt_g: 0.006,
                price_euro: 1.10,
            },
            price_per_pack: None,
            pack_weight_g: None,
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: None,
            ingredients_text: Some("Enciam, tomàquet, alvocat, oli".into()),
        },
    ];

    for ing in &mut list {
        if ing.glycemic_index.is_none() {
            ing.glycemic_index = Some(estimate_glycemic_index(&ing.name, ing.per_unit_nutrition.carbs_g, ing.ingredients_text.as_deref()));
        }
    }

    list
}

pub fn get_seed_dishes() -> Vec<Dish> {
    vec![
        Dish {
            id: "arros_amb_cigrons".into(),
            name: "Arròs amb cigrons".into(),
            description: Some("Plat principal ric en carbohidrats complexos i fibra".into()),
            items: vec![
                DishItem { ingredient_id: "ing_arros_cigrons".into(), quantity: 1.0 },
            ],
            servings: 1.0,
        },
        Dish {
            id: "amanida_propia".into(),
            name: "Amanida pròpia".into(),
            description: Some("Amanida variada amb alvocat i oli d'oliva".into()),
            items: vec![
                DishItem { ingredient_id: "ing_amanida_propia".into(), quantity: 1.0 },
            ],
            servings: 1.0,
        },
    ]
}

pub fn get_seed_weekly_menu() -> WeeklyMenu {
    let mut weekly = WeeklyMenu::default();
    
    // Day 1 (Dilluns) as configured in spreadsheet
    if let Some(dilluns) = weekly.days.get_mut(0) {
        if let Some(b) = dilluns.meals.get_mut(&MealType::Breakfast) {
            b.push(MealEntry { id: "1".into(), item_id: "cafe_amb_llet".into(), is_dish: false, quantity: 1.0 });
            b.push(MealEntry { id: "2".into(), item_id: "poma".into(), is_dish: false, quantity: 1.0 });
        }
        if let Some(l) = dilluns.meals.get_mut(&MealType::Lunch) {
            l.push(MealEntry { id: "3".into(), item_id: "arros_amb_cigrons".into(), is_dish: true, quantity: 1.0 });
            l.push(MealEntry { id: "4".into(), item_id: "bonpreu_filet_indiot".into(), is_dish: false, quantity: 3.5 });
            l.push(MealEntry { id: "5".into(), item_id: "zespri_kiwi_verd".into(), is_dish: false, quantity: 1.0 });
        }
        if let Some(d) = dilluns.meals.get_mut(&MealType::Dinner) {
            d.push(MealEntry { id: "6".into(), item_id: "amanida_propia".into(), is_dish: true, quantity: 1.0 });
            d.push(MealEntry { id: "7".into(), item_id: "llobarro".into(), is_dish: false, quantity: 2.0 });
            d.push(MealEntry { id: "8".into(), item_id: "alvocat".into(), is_dish: false, quantity: 1.4 });
        }
    }

    weekly
}
