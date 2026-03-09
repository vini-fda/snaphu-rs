#[path = "common/kumamoto.rs"]
mod kumamoto;
#[path = "common/snaphu.rs"]
mod snaphu;

use snaphu_rs::phase_compare::{gradient_compare, wrapped_error_stats};
use std::error::Error;

use crate::kumamoto::KUMAMOTO_CASES;

#[test]
fn kumamoto_real_data_matches_c_snaphu() -> Result<(), Box<dyn Error>> {
    for case in KUMAMOTO_CASES {
        if let Err(reason) = kumamoto::can_run_case(case) {
            eprintln!("skipping kumamoto test: {reason}");
            return Ok(());
        }

        let out = kumamoto::run_case(case)?;
        let wrapped_stats = wrapped_error_stats(&out.c_unwrapped, &out.rs_unwrapped);
        let grad_stats =
            gradient_compare(&out.c_unwrapped, &out.rs_unwrapped, case.width, case.height);

        eprintln!(
            "{}: wrapped rmse={:.6}, wrapped max_abs={:.6}, grad(dx,dy) wrapped-rmse=({:.6},{:.6})",
            case.name,
            wrapped_stats.rmse,
            wrapped_stats.max_abs,
            grad_stats.dx_wrapped.rmse,
            grad_stats.dy_wrapped.rmse
        );

        assert!(
            wrapped_stats.rmse < 2.0e-2,
            "{}: wrapped RMSE too high: {}",
            case.name,
            wrapped_stats.rmse
        );
        assert!(
            wrapped_stats.max_abs < 1.5e-1,
            "{}: wrapped max abs too high: {}",
            case.name,
            wrapped_stats.max_abs
        );
        assert!(
            grad_stats.dx_wrapped.rmse < 2.0e-2,
            "{}: wrapped dx gradient RMSE too high: {}",
            case.name,
            grad_stats.dx_wrapped.rmse
        );
        assert!(
            grad_stats.dy_wrapped.rmse < 2.0e-2,
            "{}: wrapped dy gradient RMSE too high: {}",
            case.name,
            grad_stats.dy_wrapped.rmse
        );
    }

    Ok(())
}
