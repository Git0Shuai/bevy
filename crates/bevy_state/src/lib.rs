#![no_std]

//! In Bevy, states are app-wide interdependent, finite state machines that are generally used to model the large scale structure of your program: whether a game is paused, if the player is in combat, if assets are loaded and so on.
//!
//! This module provides 3 distinct types of state, all of which implement the [`States`](state::States) trait:
//!
//! - Standard [`States`](state::States) can only be changed by manually setting the [`NextState<S>`](state::NextState) resource.
//!   These states are the baseline on which the other state types are built, and can be used on
//!   their own for many simple patterns. See the [states example](https://github.com/bevyengine/bevy/blob/latest/examples/state/states.rs)
//!   for a simple use case.
//! - [`SubStates`](state::SubStates) are children of other states - they can be changed manually using [`NextState<S>`](state::NextState),
//!   but are removed from the [`World`](bevy_ecs::prelude::World) if the source states aren't in the right state. See the [sub_states example](https://github.com/bevyengine/bevy/blob/latest/examples/state/sub_states.rs)
//!   for a simple use case based on the derive macro, or read the trait docs for more complex scenarios.
//! - [`ComputedStates`](state::ComputedStates) are fully derived from other states - they provide a [`compute`](state::ComputedStates::compute) method
//!   that takes in the source states and returns their derived value. They are particularly useful for situations
//!   where a simplified view of the source states is necessary - such as having an `InAMenu` computed state, derived
//!   from a source state that defines multiple distinct menus. See the [computed state example](https://github.com/bevyengine/bevy/blob/latest/examples/state/computed_states.rs)
//!   to see usage samples for these states.
//!
//! Most of the utilities around state involve running systems during transitions between states, or
//! determining whether to run certain systems, though they can be used more directly as well. This
//! makes it easier to transition between menus, add loading screens, pause games, and more.
//!
//! Specifically, Bevy provides the following utilities:
//!
//! - 3 Transition Schedules - [`OnEnter<S>`](crate::state::OnEnter), [`OnExit<S>`](crate::state::OnExit) and [`OnTransition<S>`](crate::state::OnTransition) - which are used
//!   to trigger systems specifically during matching transitions.
//! - A [`StateTransitionEvent<S>`](crate::state::StateTransitionEvent) that gets fired when a given state changes.
//! - The [`in_state<S>`](crate::condition::in_state) and [`state_changed<S>`](crate::condition::state_changed) run conditions - which are used
//!   to determine whether a system should run based on the current state.
//!
//! Bevy also provides ("state-scoped entities")[`crate::state_scoped`] functionality for managing the lifetime of entities in the context of game states.
//! This, especially in combination with system scheduling, enables a flexible and expressive way to manage spawning and despawning entities.

#![cfg_attr(
    any(docsrs, docsrs_dep),
    expect(
        internal_features,
        reason = "rustdoc_internals is needed for fake_variadic"
    )
)]
#![cfg_attr(any(docsrs, docsrs_dep), feature(rustdoc_internals))]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

// Required to make proc macros work in bevy itself.
extern crate self as bevy_state;

#[cfg(feature = "bevy_app")]
/// Provides [`App`](bevy_app::App) and [`SubApp`](bevy_app::SubApp) with state installation methods
pub mod app;
/// Provides extension methods for [`Commands`](bevy_ecs::prelude::Commands).
pub mod commands;
/// Provides definitions for the runtime conditions that interact with the state system
pub mod condition;
/// Provides definitions for the basic traits required by the state system
pub mod state;

/// Provides tools for managing the lifetime of entities based on state transitions.
pub mod state_scoped;
#[cfg(feature = "bevy_app")]
/// Provides [`App`](bevy_app::App) and [`SubApp`](bevy_app::SubApp) with methods for registering
/// state-scoped events.
pub mod state_scoped_events;

#[cfg(feature = "bevy_reflect")]
/// Provides definitions for the basic traits required by the state system
pub mod reflect;

/// The state prelude.
///
/// This includes the most common types in this crate, re-exported for your convenience.
pub mod prelude {
    #[cfg(feature = "bevy_app")]
    #[doc(hidden)]
    pub use crate::{app::AppExtStates, state_scoped_events::StateScopedEventsAppExt};

    #[cfg(feature = "bevy_reflect")]
    #[doc(hidden)]
    pub use crate::reflect::{ReflectFreelyMutableState, ReflectState};

    #[doc(hidden)]
    pub use crate::{
        commands::CommandsStatesExt,
        condition::*,
        state::{
            last_transition, ComputedStates, EnterSchedules, ExitSchedules, NextState, OnEnter,
            OnExit, OnTransition, State, StateSet, StateTransition, StateTransitionEvent, States,
            SubStates, TransitionSchedules,
        },
        state_scoped::{DespawnOnEnterState, DespawnOnExitState},
    };
}
