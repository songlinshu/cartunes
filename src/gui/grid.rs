use crate::setup::Setup;
use epaint::Galley;
use std::collections::HashMap;
use std::sync::Arc;

/// Provides structure for representing a grid of string values.
pub(crate) struct SetupGrid {
    /// Column widths are provided here.
    columns: Vec<f32>,

    /// The grid contains zero or more groups.
    groups: Vec<Group>,
}

// A group containing a matrix of strings.
struct Group {
    /// Group name is shown in a collapsible header.
    name: String,

    /// The matrix is row-major
    ///
    /// I.e. the inner vector is a list of columns with the same length as `Grid::columns`.
    matrix: Vec<Vec<Arc<Galley>>>,
}

impl SetupGrid {
    /// Create a new `SetupGrid` from a slice of `Setup`s.
    pub(crate) fn new(ui: &egui::Ui, setups: &[&Setup]) -> Self {
        // Gather groups
        let mut groups = intersect_keys(setups);
        groups.sort_unstable();

        let column_count = setups.len() + 1;
        let mut output = Self {
            columns: Vec::with_capacity(column_count),
            groups: Vec::with_capacity(groups.len()),
        };
        output.columns.resize(column_count, 0.0);

        for prop_group in groups {
            // Gather property names
            let prop_names: Vec<_> = setups
                .iter()
                .map(|setup| setup.get(prop_group).unwrap())
                .collect();
            let mut prop_names = intersect_keys(&prop_names);
            prop_names.sort_unstable();

            let mut group = Group {
                name: prop_group.to_string(),
                matrix: Vec::with_capacity(prop_names.len()),
            };

            for prop_name in prop_names {
                let mut i = 0;
                let mut columns = Vec::with_capacity(column_count);

                // Calculate width of `prop_name`
                let galley = ui
                    .fonts()
                    .layout_no_wrap(egui::TextStyle::Body, prop_name.to_string());
                let width = galley.size.x + ui.spacing().item_spacing.x * 5.0;
                output.columns[i] = output.columns[i].max(width);
                i += 1;
                columns.push(galley);

                for setup in setups {
                    let values = setup
                        .get(prop_group)
                        .unwrap()
                        .get(prop_name)
                        .unwrap()
                        .join(", ");

                    // Calculate width of `values`
                    let galley = ui.fonts().layout_no_wrap(egui::TextStyle::Body, values);
                    let width = galley.size.x + ui.spacing().item_spacing.x * 2.0;
                    output.columns[i] = output.columns[i].max(width);
                    i += 1;
                    columns.push(galley);
                }

                group.matrix.push(columns);
            }

            output.groups.push(group);
        }

        output
    }

    /// Draw the grid to the provided `Ui`.
    pub(crate) fn show(self, ui: &mut egui::Ui, car_name: &str) {
        // TODO: Colors

        let column_widths = &self.columns;

        // Draw headers
        for prop_group in self.groups.into_iter() {
            egui::CollapsingHeader::new(&prop_group.name)
                .id_source(format!("{}-{}", car_name, prop_group.name))
                .default_open(true)
                .show(ui, |ui| {
                    // Draw each row
                    for row in prop_group.matrix.into_iter() {
                        ui.horizontal(|ui| {
                            // Draw each column
                            for (i, galley) in row.into_iter().enumerate() {
                                let size = egui::Vec2::new(column_widths[i], galley.size.y);
                                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

                                ui.painter()
                                    .galley(rect.min, galley, ui.visuals().text_color());
                            }
                        });
                    }
                });
        }
    }
}

/// Get the intersection of keys that exists in each `HashMap`.
fn intersect_keys<'a, T>(maps: &'a [&'a HashMap<String, T>]) -> Vec<&'a str> {
    let mut all_keys = maps
        .iter()
        .map(|inner| inner.keys().map(|s| s.as_str()).collect());

    let mut output = if let Some(output) = all_keys.next() {
        output
    } else {
        Vec::new()
    };

    for keys in all_keys {
        output.retain(|key| keys.contains(key));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test `intersect_keys()` with two `HashMap`s.
    #[test]
    fn test_intersect_keys_two() {
        let mut map = HashMap::new();
        map.insert("foo".to_string(), ());
        map.insert("bar".to_string(), ());

        let maps = &[&map, &map];
        let keys = intersect_keys(maps);
        assert!(keys.contains(&"foo"));
        assert!(keys.contains(&"bar"));
        assert_eq!(keys.len(), 2);
    }

    /// Test `intersect_keys()` with three `HashMap`s.
    #[test]
    fn test_intersect_keys_three() {
        let mut map = HashMap::new();
        map.insert("foo".to_string(), ());
        map.insert("bar".to_string(), ());

        let maps = &[&map, &map, &map];
        let keys = intersect_keys(maps);
        assert!(keys.contains(&"foo"));
        assert!(keys.contains(&"bar"));
        assert_eq!(keys.len(), 2);
    }

    /// Test `intersect_keys()` with four `HashMap`s.
    #[test]
    fn test_intersect_keys_four() {
        let mut map = HashMap::new();
        map.insert("foo".to_string(), ());
        map.insert("bar".to_string(), ());

        let maps = &[&map, &map, &map, &map];
        let keys = intersect_keys(maps);
        assert!(keys.contains(&"foo"));
        assert!(keys.contains(&"bar"));
        assert_eq!(keys.len(), 2);
    }

    /// Test `intersect_keys()` with a "super" `HashMap` and a "sub" `HashMap`.
    ///
    /// The two maps are the same except "super" contains an additional key.
    #[test]
    fn test_intersect_keys_super_sub() {
        let mut sub_map = HashMap::new();
        sub_map.insert("foo".to_string(), ());
        sub_map.insert("bar".to_string(), ());
        let mut super_map = sub_map.clone();
        super_map.insert("baz".to_string(), ());

        let maps = &[&super_map, &sub_map];
        let keys = intersect_keys(maps);
        assert!(keys.contains(&"foo"));
        assert!(keys.contains(&"bar"));
        assert_eq!(keys.len(), 2);
    }

    /// Test `intersect_keys()` with a "sub" `HashMap` and a "super" `HashMap`.
    ///
    /// The two maps are the same except "super" contains an additional key.
    #[test]
    fn test_intersect_keys_sub_super() {
        let mut sub_map = HashMap::new();
        sub_map.insert("foo".to_string(), ());
        sub_map.insert("bar".to_string(), ());
        let mut super_map = sub_map.clone();
        super_map.insert("baz".to_string(), ());

        let maps = &[&sub_map, &super_map];
        let keys = intersect_keys(maps);
        assert!(keys.contains(&"foo"));
        assert!(keys.contains(&"bar"));
        assert_eq!(keys.len(), 2);
    }

    /// Test `intersect_keys()` with `HashMap`s that share only a few keys.
    #[test]
    fn test_intersect_keys_with_intersection() {
        let mut map_a = HashMap::new();
        map_a.insert("foo".to_string(), ());
        map_a.insert("bar".to_string(), ());
        let mut map_b = map_a.clone();
        map_a.insert("baz".to_string(), ());
        map_b.insert("quux".to_string(), ());

        let maps = &[&map_a, &map_b];
        let keys = intersect_keys(maps);
        assert!(keys.contains(&"foo"));
        assert!(keys.contains(&"bar"));
        assert_eq!(keys.len(), 2);
    }

    /// Test `intersect_keys()` with `HashMap`s that share no keys.
    #[test]
    fn test_intersect_keys_without_intersection() {
        let mut map_a = HashMap::new();
        map_a.insert("foo".to_string(), ());
        map_a.insert("bar".to_string(), ());
        let mut map_b = HashMap::new();
        map_b.insert("baz".to_string(), ());
        map_b.insert("quux".to_string(), ());

        let maps = &[&map_a, &map_b];
        let keys = intersect_keys(maps);
        assert_eq!(keys.len(), 0);
    }
}
