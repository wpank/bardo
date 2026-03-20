//! Reusable ratatui widgets for terminal screens.

#![allow(dead_code)]
#![allow(unused_imports)]

pub(crate) mod feed;
pub(crate) mod gauge;
pub(crate) mod heatmap;
pub(crate) mod key_help;
pub(crate) mod progress_bar;
pub(crate) mod scrolllist;
pub(crate) mod sparkline;
pub(crate) mod status_bar;
pub(crate) mod tabs;
pub(crate) mod timeline;

pub(crate) use feed::{EventFeed, FeedEntry, FeedLevel};
pub(crate) use gauge::{
    AccuracyGauge, ConfidenceGauge, MockPhase, VitalityGauge, vitality_to_phase,
};
pub(crate) use heatmap::{PheromoneHeatmap, PheromoneLayer};
pub(crate) use key_help::{KeyBinding, KeyHelpOverlay};
pub(crate) use progress_bar::TotalProgressBar;
pub(crate) use scrolllist::ScrollableList;
pub(crate) use sparkline::BrailleSparkline;
pub(crate) use status_bar::StatusBar;
pub(crate) use tabs::TabBar;
pub(crate) use timeline::{RibbonEventType, TimelineEvent, TimelineRibbon};
