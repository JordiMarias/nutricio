# 🍏 Food & Ingredient JSON Specification for AI Agents & Chatbots

This document describes the exact JSON structure required by **Nutrició / NutritionalPlanner** to import food ingredients into its database.

---

## 🤖 Ready-to-Use Prompt for AI Agents / Chatbots

Copy and paste the following prompt when instructing an LLM or scraping agent to generate food data for Nutrició:

````markdown
You are a nutritional data assistant. Extract and format the requested food items into valid JSON strictly conforming to the Nutrició schema.

### Output Requirements:
1. Output ONLY a valid JSON object with an `"ingredients"` array (or a JSON array of ingredient objects).
2. All numeric nutrition values are per 100g (or per unit) as floating-point numbers.
3. Use the exact enum values for `nova_group` and `unit_type`.
4. Ensure the calculated `price_euro` represents the cost per 100g (or per single unit).

### JSON Template:
```json
{
  "ingredients": [
    {
      "id": "manual_unique_id",
      "name": "Product Name in Catalan or Spanish or English",
      "brand": "Brand Name or Generic",
      "source_url": "https://example.com/product/123",
      "unit_type": "Per100g",
      "per_unit_nutrition": {
        "kcal": 350.0,
        "fat_g": 2.5,
        "saturated_fat_g": 0.5,
        "carbs_g": 70.0,
        "sugars_g": 1.5,
        "fiber_g": 6.0,
        "protein_g": 12.0,
        "salt_g": 0.02,
        "price_euro": 0.25
      },
      "price_per_pack": 2.50,
      "pack_weight_g": 1000.0,
      "nova_group": "Group1Unprocessed",
      "glycemic_index": 45,
      "ingredients_text": "Full ingredient list as printed on packaging."
    }
  ]
}
```
````

---

## 📋 Field Definitions & Schema

### Top-Level Formats
Nutrició accepts three root JSON structures:
1. **Object with `"ingredients"` array** (Recommended): `{"ingredients": [ ... ]}` (also accepts `"aliments"` or `"alimentos"`).
2. **Direct array**: `[ { ... }, { ... } ]`
3. **Full state**: `{"ingredients": [ ... ], "dishes": [ ... ]}`

---

### Ingredient Object Fields

| Field | Type | Nullable | Description & Constraints |
| :--- | :--- | :--- | :--- |
| `id` | `string` | No | Unique identifier. Use prefixes like `"base_1"`, `"manual_1"`, or a slug like `"arros_integral"`. |
| `name` | `string` | No | Full display name of the food item. |
| `brand` | `string` | Yes (`null`) | Brand or supermarket name (e.g. `"BONPREU"`, `"Hacendado"`, `"Genèric"`). |
| `source_url` | `string` | Yes (`null`) | Direct URL to product webpage, store page, or source. |
| `unit_type` | `enum / object` | No | Base unit measurement (see [Unit Type Specification](#unit-type-specification)). |
| `per_unit_nutrition` | `object` | No | Nutritional breakdown per 100g or per unit (see [Nutritional Info](#nutritional-info-specification)). |
| `price_per_pack` | `number` (float) | Yes (`null`) | Total retail price of the package in Euros (€). |
| `pack_weight_g` | `number` (float) | Yes (`null`) | Total net weight/volume of the package in grams (or mL). |
| `nova_group` | `string` | Yes (`null`) | NOVA food processing level (see [NOVA Group Specification](#nova-group-specification)). |
| `glycemic_index` | `integer` | Yes (`null`) | Glycemic Index (GI) between `0` and `100`. |
| `ingredients_text` | `string` | Yes (`null`) | Complete ingredient list string from the product label. |

---

### Unit Type Specification (`unit_type`)

- **Standard 100g / 100mL food**:
  ```json
  "unit_type": "Per100g"
  ```
- **Discrete single-unit food (e.g. eggs, burger buns, single apples)**:
  ```json
  "unit_type": {
    "PerUnit": {
      "grams_per_unit": 60.0
    }
  }
  ```

---

### Nutritional Info Specification (`per_unit_nutrition`)

All fields must be numbers (floats). For `"Per100g"` items, values are per 100 grams. For `"PerUnit"` items, values are per single unit.

| Field | Type | Unit | Description |
| :--- | :--- | :--- | :--- |
| `kcal` | `float` | kcal | Total energy / calories |
| `fat_g` | `float` | grams | Total fats |
| `saturated_fat_g` | `float` | grams | Saturated fatty acids |
| `carbs_g` | `float` | grams | Total carbohydrates |
| `sugars_g` | `float` | grams | Simple sugars |
| `fiber_g` | `float` | grams | Dietary fiber |
| `protein_g` | `float` | grams | Total protein |
| `salt_g` | `float` | grams | Salt equivalent ($2.5 \times \text{sodium}$) |
| `price_euro` | `float` | € (EUR) | Calculated cost per 100g or per single unit |

> **💡 Price Calculation Formula:**
> - For `"Per100g"`: `price_euro = (price_per_pack / pack_weight_g) * 100`
> - For `"PerUnit"`: `price_euro = price_per_pack / (pack_weight_g / grams_per_unit)`

---

### NOVA Group Specification (`nova_group`)

Must be one of the following exact string enum values:

| Enum Value | NOVA Category | Examples |
| :--- | :--- | :--- |
| `"Group1Unprocessed"` | **NOVA 1**: Unprocessed or Minimally Processed | Fresh fruits, vegetables, raw meats, fish, eggs, whole grains, raw legumes, plain milk, plain yogurt, dried legumes, tap/mineral water. |
| `"Group2ProcessedIngredient"` | **NOVA 2**: Processed Culinary Ingredients | Olive oil, vegetable oils, butter, table sugar, sea salt, vinegar, honey, culinary fats. |
| `"Group3Processed"` | **NOVA 3**: Processed Foods | Canned vegetables/legumes in brine, canned fish in oil/water, freshly baked artisanal bread, traditional cured cheese, salted nuts. |
| `"Group4UltraProcessed"` | **NOVA 4**: Ultra-Processed Food Products | Soft drinks, packaged pastries, industrial bread with emulsifiers, instant noodles, nuggets, flavored chips, foods with artificial additives, emulsifiers, sweeteners, or flavor enhancers. |

---

## 💡 Concrete Examples

### Example 1: Standard Supermarket Item (NOVA 1 - Per 100g)
```json
{
  "id": "base_101",
  "name": "BONPREU Llenties pardines cuites",
  "brand": "BONPREU",
  "source_url": "https://www.compraonline.bonpreuesclat.cat/products/bonpreu-llenties-pardines-cuites/01234",
  "unit_type": "Per100g",
  "per_unit_nutrition": {
    "kcal": 88.0,
    "fat_g": 0.5,
    "saturated_fat_g": 0.1,
    "carbs_g": 12.0,
    "sugars_g": 0.5,
    "fiber_g": 4.5,
    "protein_g": 6.8,
    "salt_g": 0.65,
    "price_euro": 0.235
  },
  "price_per_pack": 0.94,
  "pack_weight_g": 400.0,
  "nova_group": "Group3Processed",
  "glycemic_index": 30,
  "ingredients_text": "Llenties pardines, aigua, sal, segrestant (EDTA)."
}
```

### Example 2: Culinary Ingredient (NOVA 2)
```json
{
  "id": "base_102",
  "name": "Oli d'oliva verge extra",
  "brand": "BONPREU",
  "source_url": "https://www.compraonline.bonpreuesclat.cat/products/bonpreu-oli-d-oliva-verge-extra/05290",
  "unit_type": "Per100g",
  "per_unit_nutrition": {
    "kcal": 900.0,
    "fat_g": 100.0,
    "saturated_fat_g": 16.0,
    "carbs_g": 0.0,
    "sugars_g": 0.0,
    "fiber_g": 0.0,
    "protein_g": 0.0,
    "salt_g": 0.0,
    "price_euro": 0.919
  },
  "price_per_pack": 9.19,
  "pack_weight_g": 1000.0,
  "nova_group": "Group2ProcessedIngredient",
  "glycemic_index": 0,
  "ingredients_text": "100% Oli d'oliva verge extra d'extracció en fred."
}
```

### Example 3: Per-Unit Item (Eggs - 60g each)
```json
{
  "id": "base_103",
  "name": "Ous frescos classe L (dotzena)",
  "brand": "BONPREU",
  "source_url": "https://www.compraonline.bonpreuesclat.cat/products/ous-classe-l/08871",
  "unit_type": {
    "PerUnit": {
      "grams_per_unit": 63.0
    }
  },
  "per_unit_nutrition": {
    "kcal": 88.2,
    "fat_g": 6.11,
    "saturated_fat_g": 1.76,
    "carbs_g": 0.44,
    "sugars_g": 0.44,
    "fiber_g": 0.0,
    "protein_g": 7.88,
    "salt_g": 0.23,
    "price_euro": 0.22
  },
  "price_per_pack": 2.64,
  "pack_weight_g": 756.0,
  "nova_group": "Group1Unprocessed",
  "glycemic_index": 0,
  "ingredients_text": "Ous frescos de gallina."
}
```

---

## 🛠️ JSON Schema (Draft-07)

You can validate LLM outputs programmatically with this JSON Schema:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "NutritionalPlannerImport",
  "type": "object",
  "properties": {
    "ingredients": {
      "type": "array",
      "items": {
        "type": "object",
        "required": [
          "id",
          "name",
          "unit_type",
          "per_unit_nutrition"
        ],
        "properties": {
          "id": { "type": "string" },
          "name": { "type": "string" },
          "brand": { "type": ["string", "null"] },
          "source_url": { "type": ["string", "null"], "format": "uri" },
          "unit_type": {
            "oneOf": [
              { "type": "string", "enum": ["Per100g", "Per100ml"] },
              {
                "type": "object",
                "required": ["PerUnit"],
                "properties": {
                  "PerUnit": {
                    "type": "object",
                    "required": ["grams_per_unit"],
                    "properties": {
                      "grams_per_unit": { "type": "number", "minimum": 0.1 }
                    }
                  }
                }
              }
            ]
          },
          "per_unit_nutrition": {
            "type": "object",
            "required": [
              "kcal",
              "fat_g",
              "saturated_fat_g",
              "carbs_g",
              "sugars_g",
              "fiber_g",
              "protein_g",
              "salt_g",
              "price_euro"
            ],
            "properties": {
              "kcal": { "type": "number", "minimum": 0 },
              "fat_g": { "type": "number", "minimum": 0 },
              "saturated_fat_g": { "type": "number", "minimum": 0 },
              "carbs_g": { "type": "number", "minimum": 0 },
              "sugars_g": { "type": "number", "minimum": 0 },
              "fiber_g": { "type": "number", "minimum": 0 },
              "protein_g": { "type": "number", "minimum": 0 },
              "salt_g": { "type": "number", "minimum": 0 },
              "price_euro": { "type": "number", "minimum": 0 }
            }
          },
          "price_per_pack": { "type": ["number", "null"] },
          "pack_weight_g": { "type": ["number", "null"] },
          "nova_group": {
            "type": ["string", "null"],
            "enum": [
              "Group1Unprocessed",
              "Group2ProcessedIngredient",
              "Group3Processed",
              "Group4UltraProcessed",
              null
            ]
          },
          "glycemic_index": {
            "type": ["integer", "null"],
            "minimum": 0,
            "maximum": 100
          },
          "ingredients_text": { "type": ["string", "null"] }
        }
      }
    }
  }
}
```
