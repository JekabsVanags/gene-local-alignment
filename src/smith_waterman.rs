use crate::similarity_matrix::SimilarityMatrix;

pub fn smith_waterman(
    seq1: &str,              
    seq2: &str,              
    similarity_matrix: &SimilarityMatrix,
    gap_open: i32,        
    gap_extend: i32,       
) -> (i32, String, String, String) { 
    let seq1_len = seq1.len();
    let seq2_len = seq2.len();
    
    // Three scoring matrices
    let mut alignment_score_sub: Vec<Vec<i32>> = vec![vec![0; seq2_len + 1]; seq1_len + 1];
    let mut alignment_score_gap_x: Vec<Vec<i32>> = vec![vec![0; seq2_len + 1]; seq1_len + 1];
    let mut alignment_score_gap_y: Vec<Vec<i32>> = vec![vec![0; seq2_len + 1]; seq1_len + 1];
    
    // 0 = stop, 1 = from M, 2 = from Ix, 3 = from Iy
    let mut traceback: Vec<Vec<u8>> = vec![vec![0; seq2_len + 1]; seq1_len + 1];
    
    // Initializing gap matrix
    for i in 1..=seq1_len {
        alignment_score_gap_x[i][0] = -(gap_open + (i as i32) * gap_extend);
    }

    for j in 1..=seq2_len {
        alignment_score_gap_y[0][j] = -(gap_open + (j as i32) * gap_extend);
    }

    // Track the highest score in M matrix
    let mut max_score = 0;
    let mut max_i = 0;
    let mut max_j = 0;

    // Collect array of characters for O(1) access
    let s1: Vec<char> = seq1.chars().collect();
    let s2: Vec<char> = seq2.chars().collect();
    
    // Fill the matrices
    for i in 1..=seq1_len {
        for j in 1..=seq2_len {
            let char1 = s1[i-1];
            let char2 = s2[j-1];
            
            // Score from similarity matrix
            let score = similarity_matrix.get_score(char1, char2).unwrap_or(0);
            
            // M matrix
            let sub_from_sub = alignment_score_sub[i - 1][j - 1] + score;
            let sub_from_gap_x = alignment_score_gap_x[i - 1][j - 1] + score;
            let sub_from_gap_y = alignment_score_gap_y[i - 1][j - 1] + score;
            
            let mut max_val = sub_from_sub;
            let mut trace_val = 1u8; // default, came from M

            //Calculate the "arrows" in the traceback matrix A
            if sub_from_gap_x >= max_val {
                max_val = sub_from_gap_x;
                trace_val = 2; // came from Ix
            }
            if sub_from_gap_y >= max_val {
                max_val = sub_from_gap_y;
                trace_val = 3; // came from Iy
            }
            if max_val < 0 {
                max_val = 0;
                trace_val = 0; // reset
            }

            alignment_score_sub[i][j] = max_val;
            traceback[i][j] = trace_val;

            // Ix matrix
            let gap_x_from_m = alignment_score_sub[i - 1][j] - gap_open - gap_extend;
            let gap_x_extend = alignment_score_gap_x[i - 1][j] - gap_extend;
            alignment_score_gap_x[i][j] = gap_x_from_m.max(gap_x_extend).max(0);
            
            // Iy matrix
            let gap_y_from_m = alignment_score_sub[i][j - 1] - gap_open - gap_extend;
            let gap_y_extend = alignment_score_gap_y[i][j - 1] - gap_extend;
            alignment_score_gap_y[i][j] = gap_y_from_m.max(gap_y_extend).max(0);
            
            // Track maximum score
            if alignment_score_sub[i][j] > max_score {
                max_score = alignment_score_sub[i][j];
                max_i = i;
                max_j = j;
            }
        }
    }

    let (aligned_seq1, aligned_seq2, annotation) = construct_alignment(
        max_i, max_j, seq1, seq2, 
        &traceback, &alignment_score_sub, &alignment_score_gap_x, &alignment_score_gap_y,
        gap_open, gap_extend
    );

    (max_score, aligned_seq1, aligned_seq2, annotation)
}

fn construct_alignment(
    max_i: usize, 
    max_j: usize,
    seq1: &str, 
    seq2: &str, 
    traceback: &Vec<Vec<u8>>,
    score_sub: &Vec<Vec<i32>>,
    score_gap_x: &Vec<Vec<i32>>,
    score_gap_y: &Vec<Vec<i32>>,
    gap_open: i32,
    gap_extend: i32
) -> (String, String, String) {
    let s1: Vec<char> = seq1.chars().collect();
    let s2: Vec<char> = seq2.chars().collect();

    let mut aligned_seq1 = Vec::new();
    let mut aligned_seq2 = Vec::new();

    let mut i = max_i;
    let mut j = max_j;
    
    // Start in M matrix
    let mut current_matrix = 0; // 0=M, 1=Ix, 2=Iy
    
    while i > 0 && j > 0 {
        match current_matrix {
            0 => { 
                // In M matrix

                let trace = traceback[i][j];
                
                if trace == 0 {
                    break; 
                }
                
                // Consume both characters (diagonal move)
                aligned_seq1.push(s1[i-1]);
                aligned_seq2.push(s2[j-1]);
                i -= 1;
                j -= 1;
                
                // Determine which matrix we came from
                current_matrix = match trace {
                    1 => 0, // came from M, stay in M
                    2 => 1, // came from Ix, switch to Ix
                    3 => 2, // came from Iy, switch to Iy
                    _ => break,
                };
            }
            1 => { 
                // In Ix - gap in seq2

                // Check if we should stop
                if i == 0 || score_gap_x[i][j] == 0 {
                    break;
                }
                
                // Write seq1 and assign gap to seq2
                aligned_seq1.push(s1[i-1]);
                aligned_seq2.push('-');
                i -= 1;
                
                // Check if we should return to M or continue in Ix
                if i > 0 {
                    let would_open = score_sub[i][j] - gap_open - gap_extend;
                    let would_extend = score_gap_x[i][j] - gap_extend;
                    
                    // If opening is better (or equal), we came from M, so return to M
                    if would_open >= would_extend {
                        current_matrix = 0;
                    }
                }
            }
            2 => { 
                // In Iy - gap in seq1

                // Check if we should stop
                if j == 0 || score_gap_y[i][j] == 0 {
                    break;
                }
                
                // Write seq2 and assign gap to seq1
                aligned_seq1.push('-');
                aligned_seq2.push(s2[j-1]);
                j -= 1;
                
                // Check if we should return to M or continue in Iy
                if j > 0 {
                    let would_open = score_sub[i][j] - gap_open - gap_extend;
                    let would_extend = score_gap_y[i][j] - gap_extend;
                    
                    if would_open >= would_extend {
                        current_matrix = 0;
                    }
                }
            }
            _ => break,
        }
    }

    // Save alignment boundaries
    let align_start_i = i;
    let align_start_j = j;

    aligned_seq1.reverse();
    aligned_seq2.reverse();

    // Build annotation for aligned region only
    let mut annotation_aligned = String::new();
    for (c1, c2) in aligned_seq1.iter().zip(aligned_seq2.iter()) {
        if *c1 == *c2 && *c1 != '-' {
            annotation_aligned.push('|');
        } else if *c1 != '-' && *c2 != '-' {
            annotation_aligned.push(':');
        } else {
            annotation_aligned.push(' ');
        }
    }

    let aligned1 = aligned_seq1.iter().collect::<String>();
    let aligned2 = aligned_seq2.iter().collect::<String>();

    (aligned1, aligned2, annotation_aligned)
}