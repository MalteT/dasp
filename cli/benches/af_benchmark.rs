use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::NamedTempFile;

// TODO: Currently broken

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

// {b,be,e}_{5..=10}_{1..=10}_{6..=10}.{apx,apx_arg,tgf,tgf_arg}

const ICCMA_2019_INSTANCE_NAMES_SHUFFLED: &[&str] = &[
    "A-2-WS_300_16_70_30",
    "T-2-afinput_exp_cycles_depvary_step4_batch_yyy02",
    "B-3-stb_339_393",
    "B-2-stb_625_161",
    "C-1-ER_100_100_3",
    "B-2-stb_330_291",
    "B-2-afinput_exp_acyclic_depvary_step8_batch_yyy09",
    "Medium-result-b17",
    "A-3-afinput_exp_cycles_indvary2_step1_batch_yyy10",
    "B-2-stb_287_202",
    "T-2-stb_265_466",
    "T-2-afinput_exp_cycles_indvary1_step1_batch_yyy07",
    "C-1-stb_373_251",
    "T-3-afinput_exp_cycles_depvary_step3_batch_yyy08",
    "T-3-afinput_exp_cycles_indvary1_step1_batch_yyy02",
    "T-4-afinput_exp_acyclic_depvary_step8_batch_yyy09",
    "A-2-afinput_exp_acyclic_depvary_step6_batch_yyy05",
    "A-3-afinput_exp_cycles_depvary_step4_batch_yyy05",
    "A-2-stb_547_485",
    "Small-result-b28",
    "T-2-stb_339_81",
    "T-3-afinput_exp_cycles_depvary_step4_batch_yyy05",
    "Medium-result-b2",
    "n256p3q08n",
    "B-2-stb_410_118",
    "T-2-afinput_exp_acyclic_depvary_step5_batch_yyy03",
    "T-3-afinput_exp_cycles_indvary2_step1_batch_yyy04",
    "T-3-afinput_exp_cycles_indvary2_step1_batch_yyy03",
    "B-2-afinput_exp_acyclic_indvary2_step2_batch_yyy10",
    "C-1-stb_463_93",
    "C-1-grd_8034_1_2",
    "C-1-WS_200_28_70_10",
    "B-3-stb_696_368",
    "Medium-result-b24",
    "B-3-afinput_exp_acyclic_depvary_step7_batch_yyy08",
    "T-2-grd_3434_3_2",
    "B-3-stb_390_450",
    "A-2-afinput_exp_cycles_indvary1_step1_batch_yyy04",
    "T-3-afinput_exp_acyclic_indvary1_step2_batch_yyy09",
    "C-2-afinput_exp_acyclic_indvary1_step2_batch_yyy07",
    "B-3-stb_522_11",
    "B-3-stb_767_429",
    "Medium-result-b23",
    "Medium-result-b25",
    "B-2-afinput_exp_cycles_indvary2_step1_batch_yyy10",
    "T-3-afinput_exp_cycles_indvary1_step1_batch_yyy03",
    "A-3-afinput_exp_cycles_indvary1_step1_batch_yyy09",
    "B-3-stb_430_209",
    "A-3-afinput_exp_cycles_indvary1_step3_batch_yyy02",
    "A-3-afinput_exp_cycles_indvary1_step2_batch_yyy01",
    "T-4-grd_6756_4_7",
    "Medium-result-b1",
    "T-4-grd_8020_3_4",
    "A-4-afinput_exp_acyclic_depvary_step8_batch_yyy09",
    "C-2-afinput_exp_cycles_depvary_step4_batch_yyy05",
    "T-4-afinput_exp_acyclic_depvary_step7_batch_yyy08",
    "T-4-afinput_exp_acyclic_indvary1_step2_batch_yyy07",
    "B-3-WS_300_16_70_30",
    "C-2-afinput_exp_cycles_indvary2_step1_batch_yyy01",
    "A-2-stb_422_250",
    "B-3-stb_763_492",
    "T-2-afinput_exp_cycles_depvary_step4_batch_yyy03",
    "Medium-result-b16",
    "C-2-afinput_exp_cycles_depvary_step3_batch_yyy08",
    "A-1-stb_291_2",
    "A-3-afinput_exp_cycles_indvary2_step1_batch_yyy03",
    "A-3-afinput_exp_cycles_indvary1_step1_batch_yyy02",
    "T-2-afinput_exp_cycles_indvary3_step1_batch_yyy04",
    "A-1-ER_100_100_1",
    "C-2-afinput_exp_acyclic_depvary_step6_batch_yyy05",
    "T-3-afinput_exp_cycles_indvary2_step1_batch_yyy10",
    "A-2-afinput_exp_cycles_depvary_step4_batch_yyy01",
    "A-3-afinput_exp_acyclic_depvary_step6_batch_yyy04",
    "n224p5q2_ve",
    "B-2-afinput_exp_cycles_indvary2_step1_batch_yyy01",
    "B-3-WS_300_16_90_30",
    "A-2-afinput_exp_cycles_indvary3_step1_batch_yyy04",
    "C-2-stb_304_352",
    "B-3-stb_428_430",
    "B-4-stb_547_485",
    "A-4-afinput_exp_acyclic_indvary1_step2_batch_yyy07",
    "T-3-afinput_exp_cycles_indvary1_step2_batch_yyy01",
    "B-2-WS_200_20_90_50",
    "B-1-afinput_exp_acyclic_depvary_step5_batch_yyy08",
    "T-3-afinput_exp_acyclic_depvary_step6_batch_yyy04",
    "T-3-afinput_exp_cycles_depvary_step4_batch_yyy04",
    "A-2-afinput_exp_cycles_depvary_step4_batch_yyy03",
    "B-3-stb_696_412",
    "A-4-afinput_exp_acyclic_depvary_step7_batch_yyy08",
    "A-2-WS_300_16_90_30",
    "B-3-stb_457_193",
    "n320p5q2_n",
    "A-3-afinput_exp_cycles_indvary1_step1_batch_yyy03",
    "T-3-grd_8034_1_2",
    "n256p5q2_e",
    "B-3-stb_327_100",
    "A-4-grd_7891_3_9",
    "A-3-afinput_exp_acyclic_depvary_step5_batch_yyy01",
    "A-3-afinput_exp_acyclic_indvary2_step2_batch_yyy10",
    "A-3-grd_8034_1_2",
    "A-2-stb_428_430",
    "B-2-stb_291_2",
    "T-3-afinput_exp_cycles_indvary1_step3_batch_yyy02",
    "B-3-stb_380_405",
    "C-3-afinput_exp_acyclic_depvary_step7_batch_yyy08",
    "B-3-stb_590_330",
    "A-3-afinput_exp_cycles_indvary2_step1_batch_yyy04",
];

fn create_extracted_temp_file<P: AsRef<Path>>(path: &P) -> NamedTempFile {
    let mut reader = BufReader::new(File::open(path).unwrap());
    let mut target = NamedTempFile::new().unwrap();
    {
        let mut writer = BufWriter::new(&mut target);
        lzma_rs::lzma_decompress(&mut reader, &mut writer).unwrap();
    }
    target
}

fn run_5_argumentation_instances(c: &mut Criterion) {
    for instance in ICCMA_2019_INSTANCE_NAMES_SHUFFLED.iter().take(5) {
        eprintln!("{:?}", ::std::env::current_dir());
        let orig_path = format!("./benches/argumentation-frameworks/{instance}.tgf.lzma");
        let file = create_extracted_temp_file(&orig_path);
        c.bench_function(&format!("ee-st '{instance}'"), |b| {
            b.iter(|| {
                assert_cmd::Command::cargo_bin("cli")
                    .expect("Cargo binary found")
                    .args(&[
                        "--file",
                        &file.path().to_str().unwrap(), // Load file
                        "--fo",
                        "tgf", // TGF format
                        "--task",
                        "ee-st", // Execute task
                    ])
                    .unwrap()
            })
        });
    }
}

criterion_group!(benches, run_5_argumentation_instances);
criterion_main!(benches);
