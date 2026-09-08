use crate::storage::AppState;
use crate::models::*;

pub fn generate_daily_report_html(state: &AppState, day: &DailyMenu, main_title: &str, subtitle: &str) -> String {
    let mut html = String::new();

    let day_nut = day.calculate_total_nutrition(&state.ingredients, &state.dishes);
    let total_cg = day.calculate_total_glycemic_load(&state.ingredients, &state.dishes);
    let goals = &state.goals;

    html.push_str(r#"<!DOCTYPE html>
<html lang="ca">
<head>
<meta charset="UTF-8">
<title>"#);
    html.push_str(main_title);
    html.push_str(r#" - Nutrició App</title>
<style>
    body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 30px; color: #1a1a1a; background-color: #fff; line-height: 1.5; }
    h1 { color: #2e7d32; border-bottom: 2px solid #2e7d32; padding-bottom: 6px; margin-bottom: 5px; }
    h2 { color: #1565c0; margin-top: 25px; margin-bottom: 12px; border-bottom: 2px solid #1565c0; padding-bottom: 4px; }
    h3.meal-header { color: #2e7d32; background-color: #e8f5e9; padding: 6px 12px; border-left: 4px solid #2e7d32; margin-top: 20px; margin-bottom: 8px; border-radius: 2px; font-size: 15px; }
    table { width: 100%; border-collapse: collapse; margin-bottom: 15px; font-size: 13px; }
    th, td { border: 1px solid #ddd; padding: 8px 10px; text-align: left; }
    th { background-color: #f5f5f5; font-weight: bold; }
    tr:nth-child(even) { background-color: #fafafa; }
    .badge { display: inline-block; padding: 3px 8px; border-radius: 4px; font-size: 11px; font-weight: bold; color: #fff; text-shadow: 0 1px 1px rgba(0,0,0,0.2); }
    .nova-1 { background-color: #2e7d32; }
    .nova-2 { background-color: #f57c00; }
    .nova-3 { background-color: #e65100; }
    .nova-4 { background-color: #c62828; }
    .ig-low { background-color: #2e7d32; }
    .ig-med { background-color: #f57c00; }
    .ig-high { background-color: #c62828; }
    .total-row { font-weight: bold; background-color: #e8f5e9 !important; }
    .subtotal-row { font-weight: bold; background-color: #f1f8e9 !important; font-size: 12px; }
    .card { background-color: #f9f9f9; border-left: 4px solid #2e7d32; padding: 12px 16px; margin-bottom: 15px; border-radius: 2px; }
    .status-ok { color: #2e7d32; font-weight: bold; }
    .status-warn { color: #f57c00; font-weight: bold; }
    .status-alert { color: #c62828; font-weight: bold; }
    .empty-meal { font-style: italic; color: #888; font-size: 12px; margin-left: 10px; margin-bottom: 15px; }
    @media print {
        body { margin: 0; font-size: 12px; }
        .no-print { display: none; }
        .page-break { page-break-before: always; }
    }
</style>
</head>
<body>
    <div class="no-print" style="margin-bottom: 20px; text-align: right;">
        <button onclick="window.print()" style="padding: 10px 22px; background-color: #2e7d32; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: bold;">🖨️ Imprimir / Desar com a PDF</button>
    </div>

    <h1>🍏 "#);
    html.push_str(main_title);
    html.push_str("</h1>\n<p style=\"color: #666; font-size: 13px; margin-top: 0;\">");
    html.push_str(subtitle);
    html.push_str("</p>\n\n<h2>🍽️ Àpats del Dia</h2>\n");

    for meal_type in MealType::all() {
        let meal_icon = match meal_type {
            MealType::Breakfast => "🌅",
            MealType::MidMorning => "☕",
            MealType::Lunch => "🍲",
            MealType::Afternoon => "🍎",
            MealType::Dinner => "🌙",
        };

        html.push_str(&format!(
            "<h3 class=\"meal-header\">{} {}</h3>\n",
            meal_icon, meal_type.name_ca()
        ));

        let entries_opt = day.meals.get(&meal_type);
        let has_entries = entries_opt.map_or(false, |e| !e.is_empty());

        if has_entries {
            let entries = entries_opt.unwrap();
            html.push_str(r#"<table>
        <thead>
            <tr>
                <th>Element</th>
                <th>Porció / Quantitat</th>
                <th>Kcal</th>
                <th>Greixos (Sat)</th>
                <th>HdC (Sucres)</th>
                <th>Fibra</th>
                <th>Proteïna</th>
                <th>Sal</th>
                <th>Classificació NOVA</th>
                <th>Índex Glucèmic</th>
                <th>Preu (€)</th>
            </tr>
        </thead>
        <tbody>
"#);

            let mut meal_nut = NutritionalInfo::zero();

            for entry in entries {
                if entry.is_dish {
                    if let Some(dish) = state.dishes.iter().find(|d| d.id == entry.item_id) {
                        let nut = dish.calculate_total_nutrition(&state.ingredients).scale(entry.quantity);
                        meal_nut.add(&nut);

                        let nova = dish.derived_nova_group(&state.ingredients);
                        let ig = dish.derived_glycemic_index(&state.ingredients);
                        let cg = dish.calculate_glycemic_load(&state.ingredients) * entry.quantity;

                        let nova_class = match nova {
                            NovaGroup::Group1Unprocessed => "nova-1",
                            NovaGroup::Group2ProcessedIngredient => "nova-2",
                            NovaGroup::Group3Processed => "nova-3",
                            NovaGroup::Group4UltraProcessed => "nova-4",
                        };
                        let ig_class = if ig <= 55 { "ig-low" } else if ig <= 69 { "ig-med" } else { "ig-high" };

                        html.push_str(&format!(
                            "<tr><td>🍲 {}</td><td>{:.1} racions</td><td>{:.0}</td><td>{:.1}g ({:.1}g)</td><td>{:.1}g ({:.1}g) [CG {:.1}]</td><td>{:.1}g</td><td>{:.1}g</td><td>{:.2}g</td><td><span class=\"badge {}\">{}</span></td><td><span class=\"badge {}\">IG {}</span></td><td>{:.2}€</td></tr>\n",
                            dish.name, entry.quantity, nut.kcal, nut.fat_g, nut.saturated_fat_g, nut.carbs_g, nut.sugars_g, cg, nut.fiber_g, nut.protein_g, nut.salt_g, nova_class, nova.short_name_ca(), ig_class, ig, nut.price_euro
                        ));
                    }
                } else {
                    if let Some(ing) = state.ingredients.iter().find(|i| i.id == entry.item_id) {
                        let nut = ing.calculate_nutrition(entry.quantity);
                        meal_nut.add(&nut);

                        let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                        let ig = ing.get_glycemic_index();
                        let cg = ing.calculate_glycemic_load(entry.quantity);

                        let unit_str = match ing.unit_type {
                            UnitType::Per100g => format!("{:.0} g", entry.quantity),
                            UnitType::PerUnit { .. } => format!("{:.1} ut", entry.quantity),
                        };
                        let nova_class = match nova {
                            NovaGroup::Group1Unprocessed => "nova-1",
                            NovaGroup::Group2ProcessedIngredient => "nova-2",
                            NovaGroup::Group3Processed => "nova-3",
                            NovaGroup::Group4UltraProcessed => "nova-4",
                        };
                        let ig_class = if ig <= 55 { "ig-low" } else if ig <= 69 { "ig-med" } else { "ig-high" };

                        html.push_str(&format!(
                            "<tr><td>🥗 {}</td><td>{}</td><td>{:.0}</td><td>{:.1}g ({:.1}g)</td><td>{:.1}g ({:.1}g) [CG {:.1}]</td><td>{:.1}g</td><td>{:.1}g</td><td>{:.2}g</td><td><span class=\"badge {}\">{}</span></td><td><span class=\"badge {}\">IG {}</span></td><td>{:.2}€</td></tr>\n",
                            ing.name, unit_str, nut.kcal, nut.fat_g, nut.saturated_fat_g, nut.carbs_g, nut.sugars_g, cg, nut.fiber_g, nut.protein_g, nut.salt_g, nova_class, nova.short_name_ca(), ig_class, ig, nut.price_euro
                        ));
                    }
                }
            }

            html.push_str(&format!(
                "<tr class=\"subtotal-row\"><td colspan=\"2\">Subtotal {}</td><td>{:.0} Kcal</td><td>{:.1}g ({:.1}g)</td><td>{:.1}g ({:.1}g)</td><td>{:.1}g</td><td>{:.1}g</td><td>{:.2}g</td><td colspan=\"2\">-</td><td>{:.2} €</td></tr>\n",
                meal_type.name_ca(), meal_nut.kcal, meal_nut.fat_g, meal_nut.saturated_fat_g, meal_nut.carbs_g, meal_nut.sugars_g, meal_nut.fiber_g, meal_nut.protein_g, meal_nut.salt_g, meal_nut.price_euro
            ));

            html.push_str("</tbody></table>\n");
        } else {
            html.push_str("<p class=\"empty-meal\">(Sense aliments afegits per a aquest àpat)</p>\n");
        }
    }

    html.push_str(r#"
    <div style="margin-top: 25px; background-color: #e8f5e9; padding: 12px 16px; border-radius: 4px; border: 1px solid #c8e6c9;">
        <h3 style="margin: 0; color: #2e7d32;">
"#);
    html.push_str(&format!(
        "RESUM TOTAL DIARI ({}) &mdash; {:.0} Kcal &nbsp;|&nbsp; Proteïnes: {:.1}g &nbsp;|&nbsp; CG Total: {:.1} &nbsp;|&nbsp; Preu: {:.2} €",
        day.day_name, day_nut.kcal, day_nut.protein_g, total_cg, day_nut.price_euro
    ));
    html.push_str("</h3></div>\n");

    html.push_str(r#"
    <h2>📊 Resum Nutricional i Objectius Diaris</h2>

    <div class="card">
        <h3>⚡ Valor Energètic Total</h3>
        <p><strong>"#);
    html.push_str(&format!("{:.0} / {:.0} Kcal</strong> ", day_nut.kcal, goals.max_kcal));
    if day_nut.kcal >= goals.min_kcal && day_nut.kcal <= goals.max_kcal {
        html.push_str("<span class=\"status-ok\">✅ Valor energètic dins del rang recomanat</span>");
    } else if day_nut.kcal < goals.min_kcal {
        html.push_str(&format!("<span class=\"status-warn\">⚠️ Falten {:.0} Kcal per arribar al mínim recomanat</span>", goals.min_kcal - day_nut.kcal));
    } else {
        html.push_str(&format!("<span class=\"status-alert\">⚠️ Superat el màxim recomanat per {:.0} Kcal</span>", day_nut.kcal - goals.max_kcal));
    }
    html.push_str(r#"</p>
    </div>

    <div class="card">
        <h3>📊 Repartiment de Macronutrients</h3>
        <ul>
"#);
    let total_g = day_nut.total_macro_grams();
    let fat_p = day_nut.fat_pct();
    let carb_p = day_nut.carbs_pct();
    let prot_p = day_nut.protein_pct();

    html.push_str(&format!(
        "<li><strong>Greixos:</strong> {:.1}% ({:.1}g) (Rec: {:.0}-{:.0}%) - {}</li>\n",
        fat_p, day_nut.fat_g, goals.min_fat_pct, goals.max_fat_pct,
        if fat_p >= goals.min_fat_pct && fat_p <= goals.max_fat_pct { "<span class=\"status-ok\">OK</span>" } else { "<span class=\"status-warn\">Ajustar</span>" }
    ));
    html.push_str(&format!(
        "<li><strong>Hidrats de Carboni:</strong> {:.1}% ({:.1}g) (Rec: {:.0}-{:.0}%) - {}</li>\n",
        carb_p, day_nut.carbs_g, goals.min_carbs_pct, goals.max_carbs_pct,
        if carb_p >= goals.min_carbs_pct && carb_p <= goals.max_carbs_pct { "<span class=\"status-ok\">OK</span>" } else { "<span class=\"status-warn\">Ajustar</span>" }
    ));
    html.push_str(&format!(
        "<li><strong>Proteïnes:</strong> {:.1}% ({:.1}g) (Rec: {:.0}-{:.0}% | >{:.0}g) - {}</li>\n",
        prot_p, day_nut.protein_g, goals.min_protein_pct, goals.max_protein_pct, goals.min_protein_g,
        if day_nut.protein_g >= goals.min_protein_g { "<span class=\"status-ok\">OK</span>" } else { "<span class=\"status-alert\">Manca proteïna</span>" }
    ));
    html.push_str("</ul>\n    <p><small>Total Kcal de macronutrients: ");
    html.push_str(&format!("{:.0} Kcal ({:.1} g totals)</small></p>\n    </div>\n", day_nut.total_macro_kcal(), total_g));

    html.push_str(r#"
    <div class="card">
        <h3>📈 Índex i Càrrega Glucèmica (CG)</h3>
        <p><strong>Càrrega Glucèmica Total del Dia: "#);
    html.push_str(&format!("{:.1} / {:.0}</strong> - ", total_cg, goals.max_glycemic_load));
    if total_cg <= goals.max_glycemic_load {
        html.push_str("<span class=\"status-ok\">✅ Càrrega Glucèmica sota el límit recomanat</span>");
    } else {
        html.push_str("<span class=\"status-alert\">⚠️ Càrrega Glucèmica elevada</span>");
    }
    html.push_str(r#"</p>
    </div>

    <div class="card">
        <h3>🏷️ Classificació i Qualitat NOVA</h3>
        <ul>
"#);
    let nova_map = day.nova_breakdown(&state.ingredients, &state.dishes);
    let total_k = day_nut.kcal;

    for group in [NovaGroup::Group1Unprocessed, NovaGroup::Group2ProcessedIngredient, NovaGroup::Group3Processed, NovaGroup::Group4UltraProcessed] {
        let kcal_g = nova_map.get(&group).copied().unwrap_or(0.0);
        let pct = if total_k > 0.0 { (kcal_g / total_k) * 100.0 } else { 0.0 };
        let nova_class = match group {
            NovaGroup::Group1Unprocessed => "nova-1",
            NovaGroup::Group2ProcessedIngredient => "nova-2",
            NovaGroup::Group3Processed => "nova-3",
            NovaGroup::Group4UltraProcessed => "nova-4",
        };
        html.push_str(&format!(
            "<li><span class=\"badge {}\">{}</span>: {:.1}% ({:.0} Kcal)</li>\n",
            nova_class, group.short_name_ca(), pct, kcal_g
        ));
    }
    html.push_str("</ul>\n");

    let ultra_kcal = nova_map.get(&NovaGroup::Group4UltraProcessed).copied().unwrap_or(0.0);
    let ultra_pct = if total_k > 0.0 { (ultra_kcal / total_k) * 100.0 } else { 0.0 };
    if ultra_pct > goals.max_ultraprocessed_pct {
        html.push_str(&format!("<p class=\"status-alert\">⚠️ Alt contingut d'Ultraprocessats ({:.1}% de Kcal diàries)</p>", ultra_pct));
    } else {
        html.push_str("<p class=\"status-ok\">✅ Dieta neta d'ultraprocessats</p>");
    }
    html.push_str("</div>\n");

    html.push_str(r#"
    <div class="card">
        <h3>⚠️ Límits i Control d'Ingredients</h3>
        <ul>
"#);
    html.push_str(&format!(
        "<li><strong>Greixos Saturats:</strong> {:.1}g (Màx: {:.0}g) - {}</li>\n",
        day_nut.saturated_fat_g, goals.max_saturated_fat_g,
        if day_nut.saturated_fat_g <= goals.max_saturated_fat_g { "<span class=\"status-ok\">OK</span>" } else { "<span class=\"status-alert\">EXCEDIT</span>" }
    ));
    html.push_str(&format!(
        "<li><strong>Sucres:</strong> {:.1}% de macros (Màx: {:.0}%) - {}</li>\n",
        day_nut.sugar_pct_of_macros(), goals.max_sugar_macro_pct,
        if day_nut.sugar_pct_of_macros() <= goals.max_sugar_macro_pct { "<span class=\"status-ok\">OK</span>" } else { "<span class=\"status-alert\">EXCEDIT</span>" }
    ));
    html.push_str(&format!(
        "<li><strong>Fibra Alimentària:</strong> {:.1}g (Mín: {:.0}g | Ideal: {:.0}g) - {}</li>\n",
        day_nut.fiber_g, goals.min_fiber_g, goals.ideal_fiber_g,
        if day_nut.fiber_g >= goals.ideal_fiber_g { "<span class=\"status-ok\">Excel·lent</span>" } else if day_nut.fiber_g >= goals.min_fiber_g { "<span class=\"status-ok\">Acceptable</span>" } else { "<span class=\"status-warn\">Insuficient</span>" }
    ));
    html.push_str(&format!(
        "<li><strong>Sal:</strong> {:.2}g (Màx: {:.0}g) - {}</li>\n",
        day_nut.salt_g, goals.max_salt_g,
        if day_nut.salt_g <= goals.max_salt_g { "<span class=\"status-ok\">OK</span>" } else { "<span class=\"status-alert\">EXCEDIT</span>" }
    ));
    html.push_str("</ul>\n</div>\n");

    html.push_str(&format!(
        "<div class=\"card\"><h3>💶 Despesa Estimada Diària: {:.2} €</h3></div>\n",
        day_nut.price_euro
    ));

    html.push_str(r#"
</body>
</html>
"#);

    html
}

pub fn generate_daily_menu_report_html(state: &AppState, day_idx: usize) -> String {
    let day = match state.weekly_menu.days.get(day_idx) {
        Some(d) => d,
        None => return "Error: Dia no trobat".to_string(),
    };
    generate_daily_report_html(
        state,
        day,
        &format!("Informe de Menú Diari: {}", day.day_name),
        "Planificació Nutricional i Classificació NOVA",
    )
}

pub fn generate_daily_journal_report_html(state: &AppState, log: &DailyLog) -> String {
    generate_daily_report_html(
        state,
        &log.daily_menu,
        &format!("Registre Diari d'Àpats: {} ({})", log.date, log.daily_menu.day_name),
        "Seguiment Nutricional Diari i Classificació NOVA",
    )
}

pub fn generate_shopping_list_report_html(state: &AppState) -> String {
    use std::collections::HashMap;

    let mut html = String::new();

    let mut ingredient_quantities: HashMap<String, f64> = HashMap::new();
    let mut total_weekly_cost = 0.0;

    for day in &state.weekly_menu.days {
        for entries in day.meals.values() {
            for entry in entries {
                if entry.is_dish {
                    if let Some(dish) = state.dishes.iter().find(|d| d.id == entry.item_id) {
                        for item in &dish.items {
                            *ingredient_quantities.entry(item.ingredient_id.clone()).or_insert(0.0) += item.quantity * entry.quantity;
                        }
                    }
                } else {
                    *ingredient_quantities.entry(entry.item_id.clone()).or_insert(0.0) += entry.quantity;
                }
            }
        }
    }

    let mut items: Vec<(String, f64)> = ingredient_quantities.into_iter().collect();
    items.sort_by(|(id_a, _), (id_b, _)| {
        let name_a = state.ingredients.iter().find(|i| &i.id == id_a).map(|i| i.name.as_str()).unwrap_or(id_a);
        let name_b = state.ingredients.iter().find(|i| &i.id == id_b).map(|i| i.name.as_str()).unwrap_or(id_b);
        name_a.cmp(name_b)
    });

    html.push_str(r#"<!DOCTYPE html>
<html lang="ca">
<head>
<meta charset="UTF-8">
<title>Llista de la Compra - Nutrició App</title>
<style>
    body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 30px; color: #1a1a1a; background-color: #fff; line-height: 1.5; }
    h1 { color: #2e7d32; border-bottom: 2px solid #2e7d32; padding-bottom: 6px; margin-bottom: 5px; }
    table { width: 100%; border-collapse: collapse; margin-top: 20px; margin-bottom: 20px; font-size: 14px; }
    th, td { border: 1px solid #ddd; padding: 10px 12px; text-align: left; }
    th { background-color: #f5f5f5; font-weight: bold; }
    tr:nth-child(even) { background-color: #fafafa; }
    .badge { display: inline-block; padding: 3px 8px; border-radius: 4px; font-size: 11px; font-weight: bold; color: #fff; text-shadow: 0 1px 1px rgba(0,0,0,0.2); }
    .nova-1 { background-color: #2e7d32; }
    .nova-2 { background-color: #f57c00; }
    .nova-3 { background-color: #e65100; }
    .nova-4 { background-color: #c62828; }
    .total-card { background-color: #e8f5e9; border: 1px solid #c8e6c9; padding: 14px 18px; border-radius: 4px; margin-top: 20px; }
    .total-card h2 { margin: 0; color: #2e7d32; font-size: 18px; }
    @media print {
        body { margin: 0; font-size: 12px; }
        .no-print { display: none; }
    }
</style>
</head>
<body>
    <div class="no-print" style="margin-bottom: 20px; text-align: right;">
        <button onclick="window.print()" style="padding: 10px 22px; background-color: #2e7d32; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: bold;">🖨️ Imprimir / Desar com a PDF</button>
    </div>

    <h1>🛒 Llista de la Compra Setmanal</h1>
    <p style="color: #666; font-size: 13px; margin-top: 0;">Ingredients i productes necessaris per al menú planificat</p>

    <table>
        <thead>
            <tr>
                <th>Aliment</th>
                <th>Grup NOVA</th>
                <th>Quantitat Total Setmanal</th>
                <th>Cost Estimat (€)</th>
            </tr>
        </thead>
        <tbody>
"#);

    for (ing_id, total_qty) in items {
        if let Some(ing) = state.ingredients.iter().find(|i| i.id == ing_id) {
            let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
            let nova_class = match nova {
                NovaGroup::Group1Unprocessed => "nova-1",
                NovaGroup::Group2ProcessedIngredient => "nova-2",
                NovaGroup::Group3Processed => "nova-3",
                NovaGroup::Group4UltraProcessed => "nova-4",
            };

            let qty_str = match ing.unit_type {
                UnitType::Per100g => format!("{:.0} g", total_qty),
                UnitType::PerUnit { .. } => format!("{:.1} unitats", total_qty),
            };

            let nut = ing.calculate_nutrition(total_qty);
            total_weekly_cost += nut.price_euro;

            html.push_str(&format!(
                "<tr><td>🥗 {}</td><td><span class=\"badge {}\">{}</span></td><td>{}</td><td>{:.2} €</td></tr>\n",
                ing.name, nova_class, nova.short_name_ca(), qty_str, nut.price_euro
            ));
        }
    }

    html.push_str("</tbody></table>\n");
    html.push_str(&format!(
        "<div class=\"total-card\"><h2>💰 Cost Estimat Total Setmanal: {:.2} €</h2></div>\n</body>\n</html>\n",
        total_weekly_cost
    ));

    html
}
