use eframe::egui;
use rfd;
use std::fs;
mod similarity_matrix;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "LoAlFind (Local alignment finder)",
        native_options,
        Box::new(|cc| Ok(Box::new(LoAlFindApp::new(cc)))),
    )
    .expect("Failed to run native app"); // CHANGE 1: Added error handling
}

struct LoAlFindApp {
    seq1_file: Option<std::path::PathBuf>,
    seq2_file: Option<std::path::PathBuf>,
    matrix_file: Option<std::path::PathBuf>,
    gap_open: String,
    gap_extend: String,
    result: String,
    result_path: Option<std::path::PathBuf>
}

impl LoAlFindApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            seq1_file: None,
            seq2_file: None,
            matrix_file: None,
            gap_open: "1".to_string(),
            gap_extend: "1".to_string(),
            result: String::new(),
            result_path: None
        }
    }
}

impl eframe::App for LoAlFindApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // File picker for Sequence 1
                    ui.horizontal(|ui| {
                        ui.label("Sequence 1 (FASTA file):");
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("FASTA", &["fasta", "fa", "fna", "txt"])
                                .add_filter("All files", &["*"])
                                .pick_file()
                            {
                                self.seq1_file = Some(path);
                            }
                        }
                    });
                    // Selected Sequence 1 file
                    if let Some(path) = &self.seq1_file {
                        ui.label(format!("Selected: {}", path.display()));
                    } else {
                        ui.label("No file selected");
                    }
                    
                    ui.add_space(5.0);
                    
                    // File picker for Sequence 2
                    ui.horizontal(|ui| {
                        ui.label("Sequence 2 (FASTA file):");
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("FASTA", &["fasta", "fa", "fna", "txt"])
                                .add_filter("All files", &["*"])
                                .pick_file()
                            {
                                self.seq2_file = Some(path);
                            }
                        }
                    });
                    //Selected sequence 2 file
                    if let Some(path) = &self.seq2_file {
                        ui.label(format!("Selected: {}", path.display()));
                    } else {
                        ui.label("No file selected");
                    }
                    
                    ui.add_space(5.0);
                    
                    // File picker for matrix
                    ui.horizontal(|ui| {
                        ui.label("Matrix file (BLOSUM62):");
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Matrix files", &["txt", "mat"])
                                .add_filter("All files", &["*"])
                                .pick_file()
                            {
                                self.matrix_file = Some(path);
                            }
                        }
                    });
                    //Selected matrix
                    if let Some(path) = &self.matrix_file {
                        ui.label(format!("Selected: {}", path.display()));
                    } else {
                        ui.label("No file selected");
                    }
                    
                    ui.add_space(10.0);
                    
                    ui.label("Gap open:");
                    ui.text_edit_singleline(&mut self.gap_open);
                    ui.label("Gap extend:");
                    ui.text_edit_singleline(&mut self.gap_extend);
                    
                    ui.add_space(20.0);
                    
                    //Output file location
                    ui.horizontal(|ui| {
                        ui.label("Output file location:");
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .pick_folder()
                            {
                                self.result_path = Some(path);
                            }
                        }
                    });

                    //Selected output
                    if let Some(path) = &self.result_path {
                        ui.label(format!("Selected: {}", path.display()));
                    } else {
                        ui.label("No file selected");
                    }

                });

                ui.vertical(|ui| {
                    ui.label("Result:");
                    ui.text_edit_multiline(&mut self.result);
                });

            });
          
       
            ui.add_space(20.0);
            
            if ui.button("Run Alignment").clicked() {
                //Validate the files are selected
                if self.seq1_file.is_some() && self.seq2_file.is_some() && self.matrix_file.is_some() {
                    let (header1, sequence1) = read_fasta_sequence(self.seq1_file.as_ref().unwrap());
                    let (header2, sequence2) = read_fasta_sequence(self.seq2_file.as_ref().unwrap());
                    let matrix: Result<similarity_matrix::SimilarityMatrix, String> = similarity_matrix::create_similarity_matrix_from_file(self.matrix_file.as_ref().unwrap());
                    println!("{:?}", matrix);
                    self.result = format!(
                        "{}\n{}\n{}\n{}\nMatrix: {:?}\nGap h: {}\nGap g: {}",
                        header1,
                        sequence1,
                        header2,
                        sequence2,
                        self.matrix_file.as_ref().unwrap(),
                        self.gap_open,
                        self.gap_extend
                    );
                } else {
                    self.result = "Error: Please select all required files".to_string();
                }
            }
        });
    }
}

fn read_fasta_sequence(path: &std::path::PathBuf) -> (String, String) {
    let content = fs::read_to_string(path).expect("Failed to open file");

    let mut lines = content.lines();

    // Get header (first line starting with '>')
    let header = lines
        .find(|line| line.starts_with('>'))
        .unwrap_or(">unknown")
        .trim()
        .to_string();

    // Collect the rest of the lines as sequence
    let sequence = lines
        .filter(|line| !line.starts_with('>'))
        .map(|line| line.trim().replace('\r', "")) 
        .collect::<Vec<_>>()
        .join("");

    (header, sequence)
}