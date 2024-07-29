use std::fs::File;
use std::io::Write;
use dace_tests::polybench_simplify;
use hist::Hist;
use crate::tracing_ri;

pub fn trace_polybench(bench: &str, data_size: usize, cache_line_size: usize, additional_params: &[usize]){
    fn check_params(required: usize, params: &[usize]) {
        if params.len() < required {
            panic!("Not enough parameters. Required: {}, Provided: {}", required, params.len());
        }
    }

    let mut tri = match bench {
        "mvt" => polybench_simplify::mvt(data_size),
        "trisolv" => polybench_simplify::trisolv(data_size),
        "syrk" => {
            check_params(1, additional_params);
            polybench_simplify::syrk(data_size, additional_params[0])
        },
        "lu" => polybench_simplify::lu(data_size),
        "trmm_trace" => {
            check_params(1, additional_params);
            polybench_simplify::trmm_trace(data_size, additional_params[0])
        },
        "lu_affine" => polybench_simplify::lu_affine(data_size),
        "syr2d" => {
            check_params(1, additional_params);
            polybench_simplify::syr2d(data_size, additional_params[0])
        },
        "gemm" => polybench_simplify::gemm(data_size),
        "cholesky" => polybench_simplify::cholesky(data_size),
        "gramschmidt_trace" => {
            check_params(1, additional_params);
            polybench_simplify::gramschmidt_trace(data_size, additional_params[0])
        },
        "3mm" => {
            check_params(4, additional_params);
            polybench_simplify::_3mm(
                data_size,
                additional_params[0],
                additional_params[1],
                additional_params[2],
                additional_params[3]
            )
        },
        "2mm" => {
            check_params(3, additional_params);
            polybench_simplify::_2mm(
                data_size,
                additional_params[0],
                additional_params[1],
                additional_params[2]
            )
        },
        "heat_3d" => {
            check_params(1, additional_params);
            polybench_simplify::heat_3d(data_size, additional_params[0])
        },
        "convolution_2d" => {
            check_params(1, additional_params);
            polybench_simplify::convolution_2d(data_size, additional_params[0])
        },
        "symm" => {
            check_params(1, additional_params);
            polybench_simplify::symm(data_size, additional_params[0])
        },
        _ => panic!("Unknown benchmark"),
    };

    let hist = tracing_ri(&mut tri, data_size, cache_line_size);
    write_hist_to_file(&hist, "output.csv");
}

fn write_hist_to_file(hist: &Hist, file_path: &str) {
    let mut file = File::create(file_path).expect("Unable to create file");
    write!(file, "{}", hist).expect("Unable to write to file");
}



