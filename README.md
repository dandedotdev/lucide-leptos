# Lucide Leptos

[<img alt="Github" src="https://img.shields.io/badge/github-dandelion--huang/lucide--leptos-654ff0?labelColor=555555&logo=github&style=for-the-badge" height="20">](https://github.com/dandelion-huang/lucide-leptos) [<img alt="GitHub License" src="https://img.shields.io/github/license/dandelion-huang/lucide-leptos?style=for-the-badge" height="20">
](https://github.com/dandelion-huang/lucide-leptos)

This is a crate based on [Lucide](https://lucide.dev/) designed for Leptos front-end applications.

## Installation

```toml
# Cargo.toml
lucide-leptos = { git = "https://github.com/dandelion-huang/lucide-leptos.git", branch = "main", default-features = false, features = ["arrow-right"] }
```

> To improve the performance, specify the features you need.

## Usage

```rust
use lucide_leptos::ArrowRight;

#[component]
fn MyComponent() -> impl IntoView {
    view! {
        <ArrowRight />
    }
}
```
