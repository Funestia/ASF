use std::{
    collections::HashSet,
    fmt::Debug,
    mem,
    ops::{Add, AddAssign, Div, Mul, Not, Rem, Sub, SubAssign},
    sync::atomic::{self, AtomicUsize},
};

use crate::{
    armor::Armor, charms::Charm, decorations::Decoration, requirements::Requirement,
    skillpoint::SkillPoint,
};
use itertools::Itertools;
use quick_cache::sync::Cache;
use rayon::prelude::*;

pub fn do_for_each_distribution<T>(
    n: u32,
    basket_count: usize,
    f: impl Fn(&[u32]) -> Option<T>,
) -> Option<T> {
    let mut index = 0;
    let mut last_start = 1;
    let mut baskets = vec![0; basket_count];
    baskets[0] = n;
    if let Some(ret) = f(&baskets) {
        return Some(ret);
    }

    loop {
        if index == basket_count - 1 {
            let buf = mem::replace(&mut baskets[index], 0);
            if buf == n {
                break None;
            }
            index = last_start;
            while baskets[index] == 0 {
                index -= 1;
            }
            last_start = index + 1;
            baskets[last_start] = buf;
        } else {
            baskets[index] -= 1;
            baskets[index + 1] += 1;
            if let Some(ret) = f(&baskets) {
                return Some(ret);
            }
            index += 1;
        }
    }
}

pub fn scores(requirements: &[Requirement], decorations: &[Decoration]) -> Vec<f64> {
    requirements
        .iter()
        .map(|requirement| {
            decorations
                .iter()
                .map(|dec| dec.points(&requirement.name) as f64 / dec.slots() as f64)
                .max_by(f64::total_cmp)
                .unwrap_or(0.0)
        })
        .collect()
}

pub fn trim<'a, T, F>(
    parts: &'a [T],
    filter: F,
    requirements: &[Requirement],
    scores: &[f64],
    max_count: usize,
) -> (Vec<usize>, Vec<&'a T>)
where
    T: SkillPoint + Eq + std::hash::Hash + std::fmt::Debug,
    F: Fn(&T) -> bool,
{
    let mut trimmed = HashSet::new();

    //Insert best 3 slot part
    if let Some(part) = parts
        .iter()
        .enumerate()
        .filter(|(_, part)| filter(part))
        .filter(|(_, part)| part.slots() == 3)
        .max_by_key(|(_, p)| p.defence())
    {
        trimmed.insert(part);
    }

    let mut parts_vec = parts
        .iter()
        .enumerate()
        .filter(|(_, part)| filter(part))
        .collect_vec();
    if !parts_vec.is_empty() {
        loop {
            let vec_len = parts_vec.len();
            let value = parts_vec[0];
            let (_, current_part) = value;

            //get best version of current part
            let best = parts_vec
                .extract_if(.., |(_, part)| {
                    requirements
                        .iter()
                        .all(|req| part.points(&req.name) == current_part.points(&req.name))
                        && part.slots() == current_part.slots()
                })
                .max_by_key(|(_, p)| p.defence())
                .unwrap();

            //only retain parts that are better than best in at least one aspect
            parts_vec.retain_mut(|(_, part)| {
                requirements
                    .iter()
                    .any(|req| part.points(&req.name) > current_part.points(&req.name))
                    || part.slots() > current_part.slots()
            });
            parts_vec.push(best);
            if vec_len == parts_vec.len() {
                break;
            }
        }
    }

    for &pair in requirements.iter().enumerate().flat_map(|(idx, req)| {
        parts_vec.iter().max_by(|(_, a), (_, b)| {
            (a.slots() as f64 * scores[idx] + a.points(&req.name) as f64)
                .total_cmp(&(b.slots() as f64 * scores[idx] + b.points(&req.name) as f64))
        })
    }) {
        trimmed.insert(pair);
    }
    for pair in parts_vec.clone().into_iter().sorted_by_key(|(_, part)| {
        let mut score = part.slots() as f64;
        for (req_idx, req) in requirements.iter().enumerate() {
            score += part.points(&req.name) as f64 / scores[req_idx];
        }
        (score * 16.0) as i32
    }) {
        if trimmed.len() == max_count {
            break;
        }
        trimmed.insert(pair);
    }
    if trimmed.is_empty() {
        trimmed.insert((0, &parts[0]));
    }
    trimmed.into_iter().unzip()
}

/* Vec is popluated as follows:
* [0]: id
* [1]: slots
* [2...]: skill points
* [-1]: score
*/
fn createvec<T>(pieces: &[&T], requirements: &[Requirement], scores: &[f64]) -> (Vec<i32>, i32)
where
    T: SkillPoint,
{
    let scores_max = scores
        .iter()
        .cloned()
        .max_by(f64::total_cmp)
        .unwrap_or_default();
    let mut currentpieces = Vec::new();
    let mut max_score = 0f64;
    for (idx, piece) in pieces.iter().enumerate() {
        currentpieces.push(idx as i32);
        let mut score = piece.slots() as f64 * scores_max; //TODO better calculation
        currentpieces.push(piece.slots());

        for req in requirements {
            let skillpoints = piece.points(&req.name);
            currentpieces.push(skillpoints);
            score += skillpoints as f64;
        }
        max_score = max_score.max(score);
        currentpieces.push(score.ceil() as i32);
    }
    (currentpieces, max_score.ceil() as i32)
}

pub fn get_weight_for_sum<T>(target: T, summands: &[T; 3], max: &[T; 3]) -> Vec<[T; 3]>
where
    T: Add<Output = T>
        + Rem<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + From<u8>
        + Copy
        + SubAssign
        + AddAssign
        + Eq
        + PartialOrd
        + Div<Output = T>
        + Debug
        + Default,
{
    let basket_to_result = |basket: &[T; 3]| {
        let mut result = [Default::default(); 3];
        for i in 0..3 {
            let val = ((basket[i] + summands[i]) - 1.into()) / summands[i];
            result[i] = val;
            if val > 0.into() && summands[i] > 10.into() {
                return None;
            }
        }
        if result[2] > max[2]
            || result[2] + result[1] > max[1]
            || T::from(3) * result[2] + T::from(2) * result[1] + result[0] > max[0]
        {
            return None;
        }
        Some(result)
    };
    let mut index = 2;
    let mut last_start = 1;
    let mut baskets = [0.into(); 3];
    let mut results = Vec::new();

    //solved if target <= 0
    if target <= 0.into() {
        results.push(basket_to_result(&baskets).unwrap());
        return results;
    }
    baskets[2] = target;
    if let Some(result) = basket_to_result(&baskets) {
        results.push(result);
    }
    loop {
        if index == 0 {
            let buf = mem::replace(&mut baskets[index], 0.into());
            if buf == target {
                break;
            }
            index = last_start;
            while baskets[index] == 0.into() {
                index += 1;
            }
            last_start = index - 1;
            baskets[last_start] = buf;
        } else {
            let carry = ((baskets[index] - 1.into()) % summands[index]) + 1.into();
            baskets[index] -= carry;
            baskets[index - 1] += carry;
            if let Some(result) = basket_to_result(&baskets) {
                results.push(result);
            }
            index -= 1;
        }
    }
    results
}

#[derive(Debug, Clone)]
pub struct FindResult {
    pub head_index: Option<usize>,
    pub body_index: usize,
    pub arms_index: Option<usize>,
    pub waist_index: Option<usize>,
    pub legs_index: Option<usize>,
    pub charms_index: usize,
    pub decorations_count_indices: Vec<(usize, usize)>,
}

pub fn find(
    head: &[&Armor],
    body: &[&Armor],
    arms: &[&Armor],
    waist: &[&Armor],
    legs: &[&Armor],
    charms: &[&Charm],
    decorations: &[&Decoration],
    requirements: &[Requirement],
    weapon_slots: usize,
    max_results: usize,
) -> Vec<FindResult> {
    let chunksize = requirements.len() + 3;
    let mut allpieces = Vec::new();
    let mut max_scores = Vec::new();
    // layout: index, slots, points, points, ...
    let decorations_createvec =
        createvec(decorations, requirements, &vec![0.0; requirements.len()]).0;
    let scores = {
        let decorations = decorations.iter().copied().cloned().collect_vec();
        scores(requirements, &decorations)
    };
    let max_possible_score_per_slot = scores
        .iter()
        .cloned()
        .max_by(f64::total_cmp)
        .unwrap_or_default();
    let (decorations_grouped, attribute_order, decoration_summands, negatives_map) = {
        //choose a high value as default to have the least amount of unecessary calculations later
        let mut summands: Vec<[_; 3]> = vec![[127; 3]; requirements.len()];
        let mut chunked: Vec<_> = decorations_createvec.chunks_exact(chunksize).collect();
        let mut grouped = vec![Default::default(); requirements.len()];
        let mut negative_indices = vec![None; requirements.len()];
        let mut negatives = Vec::new();
        while let Some(base) = chunked.first() {
            let positive_index = base[2..chunksize - 1]
                .iter()
                .enumerate()
                .find(|(_, &v)| v > 0)
                .unwrap()
                .0;
            let negative_index = base[2..chunksize - 1]
                .iter()
                .enumerate()
                .find(|(_, &v)| v < 0)
                .map(|(o, _)| o);
            if let Some(x) = negative_index {
                negatives.push(x);
            }

            let group = chunked
                .extract_if(.., |dec| dec[2 + positive_index] > 0)
                .collect_vec();
            for dec in &group {
                summands[positive_index][dec[1] as usize - 1] = dec[2 + positive_index];
            }
            grouped[positive_index] = group;
            negative_indices[positive_index] = negative_index;
        }
        let mut order = Vec::new();
        let mut indices: HashSet<_> = (0..requirements.len()).collect();
        let mut negatives_map = Vec::new();
        while let Some(&i) = indices.iter().next() {
            indices.remove(&i);
            if !negatives.iter().contains(&i) {
                order.push(i);
                let mut positive_index = i;
                negatives_map.push(negative_indices[positive_index].is_some());
                while let Some(negative_index) = negative_indices[positive_index] {
                    positive_index = negative_index;
                    negatives_map.push(negative_indices[positive_index].is_some());
                    order.push(positive_index);
                    indices.remove(&positive_index);
                }
            }
        }
        negatives_map.resize(requirements.len(), false);
        let mut missing_indices = (0..requirements.len()).filter(|x|!order.contains(x)).collect_vec();
        order.append(&mut missing_indices);
        (grouped, order, summands, negatives_map)
    };
    let check_decorations = |final_req_points_original: &Vec<i32>, slots_at_size: &[u32; 3]| {
        let mut final_req_points = final_req_points_original.clone();
        let mut selected_posibility_idx = vec![0; requirements.len()];
        let mut possibilities = vec![Default::default(); requirements.len()];
        let start_slots = [
            (slots_at_size[0] + 2 * slots_at_size[1] + 3 * slots_at_size[2]) as i32,
            (slots_at_size[1] + slots_at_size[2]) as i32,
            slots_at_size[2] as i32,
        ];
        let mut slots_remaining = vec![start_slots; requirements.len() + 1];
        let mut depth: i32 = 0;
        let first_attribute_idx = attribute_order[depth as usize];
        possibilities[0] = get_weight_for_sum(
            final_req_points[first_attribute_idx],
            &decoration_summands[first_attribute_idx],
            &slots_remaining[0],
        );
        loop {
            if depth == -1 {
                return None;
            } else if depth as usize == requirements.len() {
                let mut result_decorations = Vec::new();
                for result_depth in 0..requirements.len() {
                    let distribution =
                        possibilities[result_depth][selected_posibility_idx[result_depth]];
                    let slot_counts = [1, 2, 3];
                    let attribute = attribute_order[result_depth];
                    for (&count, slots) in distribution
                        .iter()
                        .zip(slot_counts)
                        .filter(|(&count, _slots)| count > 0)
                    {
                        if let Some(decoration) = decorations_grouped[attribute]
                            .iter()
                            .rev() //get high value
                            .find(|dec| dec[1] == slots)
                        {
                            result_decorations.push((count as usize, decoration[0] as usize));
                        }
                    }
                }
                return Some(result_decorations);
            } else {
                //small score check
                let attribute_idx_next = *attribute_order.get(depth as usize + 1).unwrap_or(&0);
                if let Some(&current_choice) =
                    possibilities[depth as usize].get(selected_posibility_idx[depth as usize])
                {
                    let next_slots_remaining = [
                        slots_remaining[depth as usize][0]
                            - current_choice[0]
                            - 2 * current_choice[1]
                            - 3 * current_choice[2],
                        slots_remaining[depth as usize][1] - current_choice[1] - current_choice[2],
                        slots_remaining[depth as usize][2] - current_choice[2],
                    ];
                    if negatives_map[depth as usize] {
                        final_req_points[attribute_idx_next] = final_req_points_original
                            [attribute_idx_next]
                            + current_choice.iter().sum::<i32>();
                    }
                    let score_needed = attribute_order
                        .iter()
                        .skip(depth as usize + 1)
                        .map(|&index| final_req_points[index] as f64 / scores[index])
                        .filter(|&points| points > 0.0)
                        .sum::<f64>() as i32;
                    slots_remaining[depth as usize + 1] = next_slots_remaining;
                    if score_needed > next_slots_remaining[0] {
                        selected_posibility_idx[depth as usize] += 1;
                        continue;
                    }
                    if let Some(next_possibility) = possibilities.get_mut(depth as usize + 1) {
                        *next_possibility = get_weight_for_sum(
                            final_req_points[attribute_idx_next],
                            &decoration_summands[attribute_idx_next],
                            &next_slots_remaining,
                        );
                        selected_posibility_idx[depth as usize + 1] = 0;
                    }
                    depth += 1;
                } else {
                    depth -= 1;
                    if depth >= 0 {
                        selected_posibility_idx[depth as usize] += 1;
                    }
                }
            }
        }
    };
    let decoration_cache = Cache::new(3000);
    {
        let (pieces, score) = createvec(charms, requirements, &scores);
        allpieces.push(pieces);
        max_scores.push(score);
        for v in [&head, &arms, &waist, &legs, &body] {
            let (pieces, score) = createvec(v, requirements, &scores);
            allpieces.push(pieces);
            max_scores.push(score);
        }
    }
    let max_score_remaining = {
        let max_body_score = max_scores.pop().unwrap();
        //skip 1 because charms don't have torso up
        for s in max_scores.iter_mut().skip(1) {
            *s = max_body_score.max(*s);
        }
        max_scores.reverse();
        let mut v = Vec::new();
        let mut current = 0;
        for score in max_scores {
            current += score;
            v.push(current)
        }
        v.reverse();
        v
    };

    /*let mut allpieces_chunked = Vec::new();
    for v in &allpieces {
        let mut slices: Vec<_> = v.chunks_exact(chunksize).collect();
        slices.sort_by_key(|s| -s[chunksize - 1]);
        allpieces_chunked.push(slices);
    }*/
    let mut allpieces_chunked = allpieces
        .iter()
        .map(|v| {
            let mut chunks = v.chunks_exact(chunksize).collect_vec();
            chunks.sort_by_key(|s| -s[chunksize - 1]);
            chunks
        })
        .collect_vec();
    let req_points = requirements.iter().map(|req| req.points).collect_vec();
    let body_stripped = allpieces_chunked.pop().unwrap();
    let req_count = req_points.len();
    let mut final_check = Vec::new();
    for current_body_part in body_stripped.iter() {
        let mut req_points = req_points.clone();
        for pos in 0..req_count {
            req_points[pos] -= current_body_part[pos + 2];
        }
        const MAXDEPTH: usize = 4;
        let mut idx = [0; MAXDEPTH + 2];
        req_points.resize(req_count * (MAXDEPTH + 2), 0);
        let mut depth = 0;
        //last element is slots for body piece
        let mut slots_per_piece = [weapon_slots; MAXDEPTH + 3];
        slots_per_piece[6] = current_body_part[1] as usize;
        let mut slots_total = [0; MAXDEPTH + 1];
        let mut indices = [current_body_part[0]; MAXDEPTH + 2];
        let mut difficulty = [0; MAXDEPTH + 2];
        loop {
            if depth > MAXDEPTH {
                let mut slot_size_amount_available = [0; 3];
                for slot in slots_per_piece {
                    if slot > 0 {
                        slot_size_amount_available[slot - 1] += 1;
                    }
                }
                let final_req_points: Vec<_> = req_points
                    .iter()
                    .cloned()
                    .skip(req_count * depth)
                    .take(req_count)
                    .collect();
                final_check.push((slot_size_amount_available, final_req_points, indices));
                depth -= 1;
                idx[depth] += 1;
                continue;
            }
            let (current_piece, current_piece_index) =
                if idx[depth] == allpieces_chunked[depth].len() && depth != 0 {
                    //charms don't
                    //have torso up
                    (*current_body_part, -1)
                } else if idx[depth] >= allpieces_chunked[depth].len() {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    idx[depth] += 1;
                    continue;
                } else {
                    let current_piece = allpieces_chunked[depth][idx[depth]];
                    (current_piece, current_piece[0])
                };

            //pieces are sorted by score so we can skip to the end
            if depth != 0
                && difficulty[depth]
                    > current_piece[current_piece.len() - 1]
                        + max_score_remaining[depth]
                        + (slots_total[depth - 1] as f64 * max_possible_score_per_slot).ceil()
                            as i32
                && idx[depth] < allpieces_chunked[depth].len()
            {
                idx[depth] = allpieces_chunked[depth].len();
                continue;
            }

            slots_per_piece[depth] = current_piece[1] as usize;
            indices[depth] = current_piece_index;
            slots_total[depth] = current_piece[1]
                + if depth == 0 {
                    weapon_slots as i32
                } else {
                    slots_total[depth - 1]
                };

            difficulty[depth + 1] = 0;
            for i in 0..req_count {
                let points = current_piece[i + 2];
                let new_req_points = req_points[req_count * depth + i] - points;
                req_points[req_count * (depth + 1) + i] = new_req_points;
                difficulty[depth + 1] += new_req_points.max(0);
            }

            //if difficulty is higher than maximum possible remaining score, don't go deeper
            if max_score_remaining[depth]
                + (slots_total[depth] as f64 * max_possible_score_per_slot).ceil() as i32
                >= difficulty[depth + 1]
            {
                depth += 1;
                idx[depth] = 0;
            } else {
                idx[depth] += 1;
            }
        }
    }

    let result_count = AtomicUsize::new(0);
    let result = final_check
        .into_par_iter()
        .filter_map(
            |(slot_size_amount_available, mut final_req_points, indices)| {
                //stop if > desired amount of results
                if result_count.load(atomic::Ordering::Relaxed) > max_results {
                    return None;
                }

                //return if trivial
                if final_req_points.iter().max().unwrap_or(&0) <= &0 {
                    let body_index = indices[5] as usize;
                    let head_index = indices[1].try_into().ok();
                    let arms_index = indices[2].try_into().ok();
                    let waist_index = indices[3].try_into().ok();
                    let legs_index = indices[4].try_into().ok();
                    let charms_index = indices[0] as usize;
                    return Some(FindResult {
                        head_index,
                        body_index,
                        arms_index,
                        waist_index,
                        legs_index,
                        charms_index,
                        decorations_count_indices: Default::default(),
                    });
                }

                let slots_count = slot_size_amount_available[0]
                    + 2 * slot_size_amount_available[1]
                    + 3 * slot_size_amount_available[2];
                let min_slots_needed = final_req_points
                    .iter()
                    .zip(scores.iter())
                    .filter(|(&points, _score)| points > 0)
                    .fold(0f64, |sum, (&req_points, &score)| {
                        sum + req_points as f64 / score
                    })
                    .floor() as u32;
                if slots_count < min_slots_needed {
                    return None;
                }

                for &attribute_idx in &attribute_order {
                    if final_req_points[attribute_idx] <= 0 {
                        continue;
                    }
                }
                let hash_calc = |points: &Vec<i32>, slots: [u32; 3]| {
                    points
                        .iter()
                        .map(|&x| (x + 31) as u128)
                        .chain(slots.iter().map(|&x| x as u128))
                        .enumerate()
                        .map(|(idx, value)| value << (6 * idx))
                        .sum::<u128>()
                };
                let calculation_difficulty =
                    final_req_points.iter().filter(|&&p| p > 2).product::<i32>();
                let use_cache = calculation_difficulty > 80;
                if use_cache {
                    //remove negatives for caching
                    for index in negatives_map
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, neg)| neg.not().then_some(idx))
                    {
                        final_req_points[index] = final_req_points[index].max(0);
                    }
                }

                // let mut hasher = DefaultHasher::new();
                // final_req_points.hash(&mut hasher);
                // slot_size_amount_available.hash(&mut hasher);
                let (decorations_result, was_present) = if use_cache {
                    let hash = hash_calc(&final_req_points, slot_size_amount_available);
                    let result = decoration_cache.get(&hash);
                    let was_present = result.is_some();
                    let result = result.unwrap_or_else(|| {
                        let value =
                            check_decorations(&final_req_points, &slot_size_amount_available);
                        decoration_cache.insert(hash, value.clone());
                        value
                    });
                    (result, was_present)
                } else {
                    (
                        check_decorations(&final_req_points, &slot_size_amount_available),
                        true,
                    )
                };

                if !was_present && calculation_difficulty > 200 {
                    decoration_cache.insert(
                        hash_calc(
                            &final_req_points,
                            [
                                slot_size_amount_available[0].saturating_sub(1),
                                slot_size_amount_available[1],
                                slot_size_amount_available[2],
                            ],
                        ),
                        None,
                    );
                    if slot_size_amount_available[1] > 0 {
                        decoration_cache.insert(
                            hash_calc(
                                &final_req_points,
                                [
                                    slot_size_amount_available[0],
                                    slot_size_amount_available[1] - 1,
                                    slot_size_amount_available[2],
                                ],
                            ),
                            None,
                        );
                        decoration_cache.insert(
                            hash_calc(
                                &final_req_points,
                                [
                                    slot_size_amount_available[0] + 1,
                                    slot_size_amount_available[1] - 1,
                                    slot_size_amount_available[2],
                                ],
                            ),
                            None,
                        );
                        decoration_cache.insert(
                            hash_calc(
                                &final_req_points,
                                [
                                    slot_size_amount_available[0] + 2,
                                    slot_size_amount_available[1] - 1,
                                    slot_size_amount_available[2],
                                ],
                            ),
                            None,
                        );
                    }
                    if slot_size_amount_available[2] > 0 {
                        decoration_cache.insert(
                            hash_calc(
                                &final_req_points,
                                [
                                    slot_size_amount_available[0] + 2,
                                    slot_size_amount_available[1],
                                    slot_size_amount_available[2] - 1,
                                ],
                            ),
                            None,
                        );
                        decoration_cache.insert(
                            hash_calc(
                                &final_req_points,
                                [
                                    slot_size_amount_available[0] + 1,
                                    slot_size_amount_available[1] + 1,
                                    slot_size_amount_available[2] - 1,
                                ],
                            ),
                            None,
                        );
                    }
                }
                decorations_result.map(|ret_decorations| {
                    let body_index = indices[5] as usize;
                    let head_index = indices[1].try_into().ok();
                    let arms_index = indices[2].try_into().ok();
                    let waist_index = indices[3].try_into().ok();
                    let legs_index = indices[4].try_into().ok();
                    let charms_index = indices[0] as usize;
                    FindResult {
                        head_index,
                        body_index,
                        arms_index,
                        waist_index,
                        legs_index,
                        charms_index,
                        decorations_count_indices: ret_decorations,
                    }
                })
            },
        )
        .take_any(max_results)
        .collect();
    result
}
