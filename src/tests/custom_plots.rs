use crate::plot::{CustomPlot, Plot, ZLayer};

const VALID: &str = r"---------------
------xxx------
------xox------
------xxx------
---------------

---------------
------xxx------
------xxx------
-----xxxxx-----
---------------";

const NO_ORIGIN: &str = r"---------------
------xxx------
------xxx------
------xxx------

---------------
------xxx------
------xxx------
---------------";

const INVALID_X_SPACE: &str = r"---------------
------xxx------
------xox-----
------xxx------

---------------
------xxx------
------xxx------
---------------";

const INVALID_Y_SPACE: &str = r"---------------
------xxx------
------xox------

---------------
------xxx------
------xxx------
---------------";

#[test]
fn create_custom_plot_from_valid_string() {
    let plot = CustomPlot::from_string(VALID, 0.1, [0., 0., 0.]);
    assert!(plot.is_ok());
}

#[test]
fn point_count_does_not_match_x_space() {
    let plot = CustomPlot::from_string(INVALID_X_SPACE, 0.1, [0., 0., 0.]);
    assert_eq!(
        plot.err().unwrap(),
        "Point count does not match space.".to_string()
    );
}

#[test]
fn point_count_does_not_match_y_space() {
    let plot = CustomPlot::from_string(INVALID_Y_SPACE, 0.1, [0., 0., 0.]);
    assert_eq!(
        plot.err().unwrap(),
        "Point count does not match space.".to_string()
    );
}

#[test]
fn create_custom_plot_from_without_origin() {
    let plot = CustomPlot::from_string(NO_ORIGIN, 0.1, [0., 0., 0.]);
    assert_eq!(
        plot.err().unwrap(),
        "First layer must have an origin.".to_string()
    );
}
