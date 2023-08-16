/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use glib::subclass::types::ObjectSubclass;
use gstreamer::gst_plugin_define;
use servowebsrc::ServoWebSrc;

mod config;
mod logging;
mod resources;
mod servowebsrc;

gst_plugin_define!(
    servoplugin,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
    "MPL",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY"),
    env!("BUILD_REL_DATE")
);

fn plugin_init(plugin: &gstreamer::Plugin) -> Result<(), glib::BoolError> {
    // Make any changes we might want to against the global servo config `Opts`, but only once.
    // This function is guarded with an `std::sync::Once` to make sure we don't re-override things
    // unintentionally later.
    config::init();

    gstreamer::gst_debug!(logging::CATEGORY, "Initializing logging");
    log::set_logger(&logging::LOGGER).expect("Failed to set logger");
    log::set_max_level(log::LevelFilter::Debug);

    log::debug!("Initializing resources");
    resources::init();

    log::debug!("Registering plugin");
    gstreamer::Element::register(
        Some(plugin),
        "servowebsrc",
        gstreamer::Rank::None,
        ServoWebSrc::get_type(),
    )
}
