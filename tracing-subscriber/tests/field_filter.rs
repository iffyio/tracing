mod support;
use self::support::*;
use tracing::{self, subscriber::with_default, Level};
use tracing_subscriber::{filter::Filter, prelude::*};

#[test]
fn field_filter_events() {
    let filter: Filter = "[{thing}]=debug".parse().expect("filter should parse");
    let (subscriber, finished) = subscriber::mock()
        .event(
            event::mock()
                .at_level(Level::INFO)
                .with_fields(field::mock("thing")),
        )
        .event(
            event::mock()
                .at_level(Level::DEBUG)
                .with_fields(field::mock("thing")),
        )
        .done()
        .run_with_handle();
    let subscriber = subscriber.with(filter);

    with_default(subscriber, || {
        tracing::trace!(disabled = true);
        tracing::info!("also disabled");
        tracing::info!(thing = 1);
        tracing::debug!(thing = 2);
        tracing::trace!(thing = 3);
    });

    finished.assert_finished();
}

#[test]
fn field_filter_spans() {
    let filter: Filter = "[{enabled=true}]=debug"
        .parse()
        .expect("filter should parse");
    let (subscriber, finished) = subscriber::mock()
        .enter(span::mock().named("span1"))
        .event(
            event::mock()
                .at_level(Level::INFO)
                .with_fields(field::mock("something")),
        )
        .exit(span::mock().named("span1"))
        .enter(span::mock().named("span2"))
        .exit(span::mock().named("span2"))
        .enter(span::mock().named("span3"))
        .event(
            event::mock()
                .at_level(Level::DEBUG)
                .with_fields(field::mock("something")),
        )
        .exit(span::mock().named("span3"))
        .done()
        .run_with_handle();
    let subscriber = subscriber.with(filter);

    with_default(subscriber, || {
        tracing::trace!("disabled");
        tracing::info!("also disabled");
        tracing::info_span!("span1", enabled = true).in_scope(|| {
            tracing::info!(something = 1);
        });
        tracing::debug_span!("span2", enabled = false, foo = "hi").in_scope(|| {
            tracing::warn!(something = 2);
        });
        tracing::trace_span!("span3", enabled = true, answer = 42).in_scope(|| {
            tracing::debug!(something = 2);
        });
    });

    finished.assert_finished();
}