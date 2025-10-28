# macOS Notification APIs

## NSUserNotificationCenter (Deprecated in macOS 11)

```rust
// Legacy API - deprecated but still functional
use objc2_foundation::{NSUserNotificationCenter, NSUserNotification};

// Clear all delivered notifications
let center = NSUserNotificationCenter::defaultUserNotificationCenter();
center.removeAllDeliveredNotifications();

// Clear specific notification
center.removeDeliveredNotification(&notification);
```

## UNUserNotificationCenter (Modern API - macOS 10.14+)

```rust
// Modern API for notification management
use objc2_user_notifications::{UNUserNotificationCenter, UNNotificationRequest};

// Get current notification center
let center = UNUserNotificationCenter::currentNotificationCenter();

// Remove all pending notifications
center.removeAllPendingNotificationRequests();

// Remove all delivered notifications  
center.removeAllDeliveredNotifications();

// Remove specific notifications by identifier
let identifiers = vec!["notification-id-1", "notification-id-2"];
center.removePendingNotificationRequestsWithIdentifiers(&identifiers);
center.removeDeliveredNotificationsWithIdentifiers(&identifiers);
```

## Key Differences

- **NSUserNotificationCenter**: Deprecated, simpler API, limited features
- **UNUserNotificationCenter**: Modern, rich features, better control

## Implementation Strategy

1. Check macOS version to determine which API to use
2. Use UNUserNotificationCenter for macOS 10.14+
3. Fall back to NSUserNotificationCenter for older versions
4. Handle permissions properly for UNUserNotificationCenter

## Auto-reload Settings

For `should_auto_reload()`, need to:
1. Check NSUserDefaults for app preferences
2. Look for specific key like "AutoReloadEnabled"
3. Return actual boolean value, not hardcoded true