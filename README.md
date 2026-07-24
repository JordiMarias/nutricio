# 🍏 Nutritional Planner

[![CI & Build Linux Binary](https://github.com/JordiMarias/NutritionalPlanner/actions/workflows/ci.yml/badge.svg)](https://github.com/JordiMarias/NutritionalPlanner/actions/workflows/ci.ymll)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui%200.29-blue.svg)](https://github.com/emilk/egui)
[![Target](https://img.shields.io/badge/target-Desktop%20%7C%20WASM-green.svg)](#)

A comprehensive, cross-platform nutritional calculator, food expenditure planner, and NOVA food classification app written in **Rust** using **`egui` / `eframe`**. It compiles seamlessly to native desktop executables (Linux, Windows, macOS) and WebAssembly (WASM) for web deployment.

---

## 🌟 Overview

**Nutrició** allows users to design balanced daily and weekly meal menus while tracking energy intake, macronutrients, micronutrients, food budget costs, Glycemic Load (GL), and food processing levels according to the **NOVA Classification System**.

It bridges nutritional science with budget management, helping users reach health targets while keeping grocery costs transparent.

---

## ✨ Key Features

- 📅 **Weekly Menu Planner**:
  - Organize meals across all 7 days of the week (Monday through Sunday) for 4 daily meal slots (Breakfast, Lunch, Afternoon Snack / *Berenar*, Dinner).
  - Real-time calculation of daily and weekly totals:
    - **Energy (Kcal)** (Target: 2200 - 2500 Kcal)
    - **Fats & Saturated Fats** (Target: 20-25% macro ratio, Saturated Fat $\le$ 19g)
    - **Carbohydrates & Sugars** (Target: 45-55% macro ratio, Sugars $\le$ 10% macro ratio)
    - **Dietary Fiber** (Target: $\ge$ 30g)
    - **Protein** (Target: 20-25% macro ratio)
    - **Salt** (Target: $\le$ 5g)
    - **Marginal Budget (€)**
  - Dynamic visual target status indicators (**Optimal**, **Warning**, **Critical**).

- 🍏 **NOVA Classification System**:
  - Classifies foods from **NOVA 1** (Unprocessed / Minimally Processed) to **NOVA 4** (Ultra-Processed Foods).
  - Automatically calculates weekly percentage of ultra-processed foods and alerts when exceeding recommended limits ($\le$ 10%).

- 📊 **Glycemic Index (GI) & Glycemic Load (GL)**:
  - Tracks Glycemic Index for ingredients and computes daily Glycemic Load (GL) per meal and per day to support metabolic and blood glucose management.

- 🥦 **Ingredient Catalog & Web Scraper**:
  - Add ingredients manually or automatically scrape items directly from **Bonpreu supermarket URLs**.
  - The embedded scraper extracts product name, price per unit/kg/L, and full per-100g nutritional facts.
  - Supports density volume-to-mass conversions (e.g. liquids in mL).

- 🍲 **Recipe & Dish Editor**:
  - Combine multiple raw ingredients with custom gram/mL portions to create reusable dishes.
  - Automatically aggregates macro profiles, NOVA classification (inherits highest NOVA tier), Glycemic Load, and cost per dish.

- 🛒 **Automated Shopping List & Cost Estimation**:
  - Computes exact ingredient requirements for the scheduled menu.
  - Generates an itemized grocery shopping list with total estimated cost (€).

- 📄 **HTML & JSON Export/Import**:
  - Export beautiful, self-contained standalone HTML menu reports (`menu_setmanal_report.html`) complete with CSS styling and summary tables.
  - Save and load complete app state as JSON (`nutricio_estat.json`).

---

## 🛠️ Technology Stack

- **Language**: [Rust](https://www.rust-lang.org/) (2021 Edition)
- **GUI Framework**: [`egui`](https://github.com/emilk/egui) / [`eframe`](https://docs.rs/eframe/latest/eframe/) (v0.29)
- **Networking & Scraping**: [`reqwest`](https://docs.rs/reqwest) (with `blocking` on desktop and native `fetch` on `wasm32`)
- **Serialization**: [`serde`](https://serde.rs/) & [`serde_json`](https://docs.rs/serde_json)
- **File Dialogs**: [`rfd`](https://github.com/emilk/rfd) (Rusty File Dialogs)
- **Targets**:
  - Desktop Native (`x86_64` Linux, Windows, macOS)
  - WebAssembly (`wasm32-unknown-unknown`) via Trunk / eframe WebRunner

---

## 🚀 Getting Started

### Prerequisites

#### Linux (Ubuntu/Debian) Dependencies
Build tools and graphics library development packages are required for `egui` and `eframe`:

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libgtk-3-dev \
  libsoup-3.0-dev
```

---

### Running the Desktop App

1. **Clone the repository**:
   ```bash
   git clone https://github.com/USER/nutricio.git
   cd nutricio
   ```

2. **Run in development mode**:
   ```bash
   cargo run
   ```

3. **Build optimized release binary**:
   ```bash
   cargo build --release
   ```
   The generated executable will be placed in `target/release/NutritionalPlanner`.

---

### Running Unit & Integration Tests

Run the full test suite (includes parsing, scraping tests, NOVA classification, and spreadsheet calculation verifications):

```bash
cargo test
```

---

### 🌐 Building for WebAssembly (WASM)

To build and serve the application as a Web App using [Trunk](https://trunkrs.dev/):

1. **Install WASM target & Trunk**:
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo install trunk
   ```

2. **Serve locally**:
   ```bash
   trunk serve
   ```
   Open `http://127.0.0.1:8080` in your web browser.

---

## ⚙️ Continuous Integration (CI) & Automated Releases

This repository includes a GitHub Actions workflow located at [`.github/workflows/ci.yml`](.github/workflows/ci.yml).

### Multi-Platform Build Matrix
The pipeline automatically compiles for:
- 🐧 **Linux x86_64 (GNU Desktop)**: `NutritionalPlanner-linux-x86_64`
- 🤖 **Linux ARM64 / AArch64 (GNU Desktop)**: `NutritionalPlanner-linux-aarch64` (Raspberry Pi, ARM SBCs)
- 🪟 **Windows x86_64 (MSVC Native)**: `NutritionalPlanner-windows-x86_64.exe`


### 📦 Automated Releases via Git Tags
Pushing a git tag formatted as `v*` (e.g. `v1.0.0`) automatically creates a **GitHub Release** and attaches binaries for all target platforms:

```bash
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```



---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

