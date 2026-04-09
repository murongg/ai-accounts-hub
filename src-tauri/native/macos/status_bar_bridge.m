#import "AAHStatusBarBridge.h"
#import <AppKit/AppKit.h>

extern bool aah_status_bar_bridge_swift_initialize(AAHStatusBarBridgeCallback callback);
extern void aah_status_bar_bridge_swift_set_payload(const char *payload_json);
extern int aah_status_bar_bridge_swift_optional_string_length_from_json(const char *payload_json, const char *field_name);
extern int aah_status_bar_bridge_swift_icon_ready(void);
extern int aah_status_bar_bridge_swift_section_count_from_json(const char *payload_json);
extern int aah_status_bar_bridge_swift_selected_tab_value_from_json(const char *payload_json);

bool aah_status_bar_bridge_initialize(AAHStatusBarBridgeCallback callback) {
  return aah_status_bar_bridge_swift_initialize(callback);
}

void aah_status_bar_bridge_set_payload(const char *payload_json) {
  aah_status_bar_bridge_swift_set_payload(payload_json);
}

int aah_status_bar_bridge_needs_main_queue_hop(int is_main_thread) {
  return is_main_thread == 0 ? 1 : 0;
}

double aah_status_bar_bridge_panel_height_for_content_height(double content_height) {
  return MAX(content_height, 0.0);
}

double aah_status_bar_bridge_panel_height_clamped_to_available_height(double content_height, double available_height) {
  double naturalHeight = aah_status_bar_bridge_panel_height_for_content_height(content_height);
  if (available_height <= 0) {
    return naturalHeight;
  }

  return MIN(naturalHeight, available_height);
}

int aah_status_bar_bridge_optional_string_length_from_json(const char *payload_json, const char *field_name) {
  return aah_status_bar_bridge_swift_optional_string_length_from_json(payload_json, field_name);
}

int aah_status_bar_bridge_debug_icon_ready(void) {
  return aah_status_bar_bridge_swift_icon_ready();
}

int aah_status_bar_bridge_debug_section_count_from_json(const char *payload_json) {
  return aah_status_bar_bridge_swift_section_count_from_json(payload_json);
}

int aah_status_bar_bridge_debug_selected_tab_value_from_json(const char *payload_json) {
  return aah_status_bar_bridge_swift_selected_tab_value_from_json(payload_json);
}
