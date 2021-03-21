use crate::genetics::Flavor;

#[derive(Debug)]
pub struct Cookbook {
    recipes: Vec<Recipe>
}

impl Cookbook {
    pub fn new() -> Cookbook {
        let pizza = Recipe::new(String::from("pizza"), Flavor::Salty);
        let ice_cream = Recipe::new(String::from("ice cream"), Flavor::Sweet);
        let mushroom = Recipe::new(String::from("mushrooms"), Flavor::Umami);
        let lemon = Recipe::new(String::from("lemon"), Flavor::Sour);

        Cookbook { 
            recipes: vec![pizza, ice_cream, mushroom, lemon],
        }
    }

    pub fn recipes(&self) -> std::slice::Iter<Recipe> {
        self.recipes.iter()
    }
}

#[derive(Debug)]
pub struct Recipe {
    name: String,
    flavor: Flavor,
}

impl Recipe {
    pub fn new(name: String, flavor: Flavor) -> Recipe {
        Recipe {name, flavor}
    }

    pub fn flavor(&self) -> Flavor {
        self.flavor
    }
}