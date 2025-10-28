# Objective-C NSMenu Target-Action Pattern with objc2

## Key Concepts from objc2 Documentation

### Target-Action Pattern Requirements

1. **Controller Creation**: Use `define_class!` macro to create NSObject subclass
2. **Thread Safety**: Specify `MainThreadOnly` as `ClassType::ThreadKind`
3. **Method Implementation**: Use `extern_methods!` for action methods

## Implementation Pattern

```rust
use objc2::{define_class, extern_methods, ClassType, MainThreadMarker};
use objc2_foundation::{NSObject, NSString};
use objc2_app_kit::{NSMenu, NSMenuItem};

// Define controller class to handle menu actions
define_class!(
    #[unsafe(super(NSObject))]
    pub struct MenuController;
);

impl ClassType for MenuController {
    type ThreadKind = MainThreadOnly;
}

impl MenuController {
    extern_methods!(
        // Action method that NSMenuItem will call
        #[method(menuItemAction:)]
        pub fn menu_item_action(&self, sender: &NSMenuItem);
    );
}

// Implementation of action method
impl MenuController {
    fn menu_item_action(&self, sender: &NSMenuItem) {
        // Get menu item tag to identify which item was clicked
        let tag = sender.tag();
        // Call appropriate Rust callback based on tag
    }
}
```

## Menu Creation with Target-Action

```rust
// Create controller instance
let controller = MenuController::new();

// Create menu item with target and action
let menu_item = NSMenuItem::new();
menu_item.setTarget(Some(&controller));
menu_item.setAction(Some(sel!(menuItemAction:)));
menu_item.setTag(item_index as i64);
```

## Key Requirements

1. **Selector Registration**: Use `sel!()` macro for action selectors
2. **Tag-based Identification**: Use `setTag()` to identify menu items
3. **Controller Lifetime**: Ensure controller lives as long as menu
4. **Main Thread**: All menu operations must be on main thread