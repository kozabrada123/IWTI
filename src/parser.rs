
use std::{fs, env, process::exit, collections::HashMap, thread, time};
use reqwest::blocking::Response;
use serde_json::{self, Value};
use serde::{Serialize, Deserialize};
use substring::{self, Substring};


// Api format structs
// -------------------------------------
#[derive(Serialize,Deserialize, Debug)]
struct ApiResult {
    iteminfo: ItemInfo,
}

// See https://github.com/csgofloat/inspect

#[derive(Serialize,Deserialize, Debug)]
struct ItemInfo {
    defindex: usize, // Weapon id ==> "Weapon ID"
    paintindex: usize, // Paint id ==> "Paint Kit"
    paintseed: usize, // Paint seed ==> "Seed"
    floatvalue: f64, // Float f64 (0-1) ==> "Wear"
    weapon_type: String, // Name of the weapon. I. E. AWP ==> "Item Name"
    item_name: String, // Name of the paint kit (skin) ==> "Paint Kit Name"
    //wear_name: String, // Wear range (I.E. FN, WW, BS..)
    full_item_name: String, // Full name of the item (I.E. AWP | Medusa (Well-Worn)))
    stickers: Vec<Sticker> // Stickers ==> "Stickers"
}

#[derive(Serialize,Deserialize, Debug)]
struct Sticker {
    stickerId: usize, // Sticker's id ==> "Sticker ID"
    slot: u8, // Its slot (0-3) ==> "Slot"
    codename: String, // In game name
    name: String, // Human readable name

}
// -------------------------------------

// Osiris format structs
// -------------------------------------
#[derive(Serialize,Deserialize, Debug)]
struct Item {
    #[serde(rename = "Item Name")]
    ItemName: String, // Weapon Name. ==> weapon_type

    #[serde(rename = "Name Tag")]
    NameTag: Option<String>, // The name tag of the item, if there is one

    #[serde(rename = "Paint Kit")]
    PaintKit: usize, // Paint kit id. ==> paintindex

    #[serde(rename = "Paint Kit Name")]
    PaintKitName: String, // Paint kit name. ==> item_name

    Seed: usize, // Paint rng seed. ==> paintseed

    Stickers: Vec<OSticker>, // Vec of Stickers.

    #[serde(rename = "Weapon ID")]
    WeaponID: usize, // The Weapon's ID. ==> defindex

    Wear: f64 // The weapon's wear or float. ==> floatvalue
}

impl Item {
    pub fn from_api(from: ItemInfo) -> Self {
        // Convert real quick
        // Construct stickers.
        let mut stickers = Vec::<OSticker>::new();

        // Convert every sticker there to a sticker here
        for fsticker in from.stickers.iter() {
            stickers.push(OSticker { StickerID: fsticker.stickerId, Slot: fsticker.slot });
        }

        // Convert the rest
        Self { ItemName: from.weapon_type, PaintKit: from.paintindex, PaintKitName: from.item_name, Seed: from.paintseed, Stickers: stickers, WeaponID: from.defindex, Wear: from.floatvalue, NameTag: None }
    }
}

#[derive(Serialize,Deserialize, Debug)]
struct OSticker {
    #[serde(rename = "Sticker ID")]
    StickerID: usize, // Sticker's id ==> stickerId
    Slot: u8, // Its slot (0-3) ==> "slot"

}
// -------------------------------------

// Steam format structs
// -------------------------------------
#[derive(Deserialize, Debug, Clone)]
struct SteamApi {
    success: bool,
    rgInventory: HashMap<String, rgInventory>,
    rgDescriptions: HashMap<String, rgDescription> // Main json that gives us the descriptions
}


#[derive(Deserialize, Debug, Clone)]
struct rgDescription { // An item
    appid: String,
    classid: String,
    instanceid: String, // Id that we need to check with rginv
    fraudwarnings: Option<Vec<String>>, // 0 is usually the name tag
    actions: Option<Vec<action>> // Its actions
}

impl rgDescription {
    fn blank() -> Self {
        Self { appid: "NaN".to_string(), classid: "NaN".to_string(), instanceid: "NaN".to_string(), fraudwarnings: None, actions: None  }
    }
}


#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
struct rgInventory { // An item in the rgInventory
    instanceid: String,
    classid: String,
    id: String // Only thing we care about
}

impl rgInventory {
    fn blank(self) -> Self {
        Self { instanceid: "NaN".to_string(), classid: "NaN".to_string(), id: "NaN".to_string() }
    }
}


#[derive(Deserialize, Debug, Clone)]
struct action {
    name: String,
    link: String // The only thing we actually need
}
// -------------------------------------

// Turns the json inspect object into osiris format
pub fn parse(input: serde_json::Value, name: Option<String>) -> String {

    // Check if we even have an iteminfo..
    if input.clone().get("iteminfo").is_none() {
        println!("Invalid item? (No iteminfo)");
        panic!();
    }

    // 3, 2, 1, SERDE!
    let apif: ApiResult = serde_json::from_value(input).unwrap();

    // Convert to Osiris
    let mut osiris = Item::from_api(apif.iteminfo);

    // If we have a name
    if name.is_some() {
        osiris.NameTag = name;
    }

    return serde_json::to_string(&osiris).unwrap();
}

pub fn get_item(url: &String, name: Option<String>) -> String {

    // Contact the api
    let resp = reqwest::blocking::get(format!("https://api.csgofloat.com/?url={}", url)).unwrap();
    
    // Assuming nothing died..
    let resp_json: serde_json::Value = serde_json::from_str(&resp.text().unwrap()).unwrap();

    // Test if we errored
    if resp_json.get("error").is_some() {
        // We have an api error
        println!("Api Error: {} ({}, Status: {}) [{}]", resp_json.get("error").unwrap(), resp_json.get("code").unwrap(), resp_json.get("status").unwrap(), format!("https://api.csgofloat.com/?url={}", url));
        
        return "".to_string();
    }

    //println!("Got weapon data..");
    //println!("Converting to osiris format..");

    // Nope, we good
    // Parse it
    let mut writedata = parse(resp_json.clone(), name);

    // Just add a comma and \n at the end
    writedata.push_str(",\n");

    return writedata;
}

// Turns the json inventory object into a String of a lot of osiris formats
// https://steamcommunity.com/id/koza1brada/inventory/json/730/2
pub fn get_inv(profile: &String, id: &String, ilimit: usize, retry: bool, customlink: bool) -> String {

    let mut limit = ilimit;

    let mut writedata = "".to_string();

    let mut resp: Response = reqwest::blocking::get("https://google.com/").unwrap();

    if customlink {
        // Contact the 
        // With the id
        resp = reqwest::blocking::get(format!("https://steamcommunity.com/id/{}/inventory/json/730/2", profile)).unwrap();
    }

    else {
        // With the custom username
        resp = reqwest::blocking::get(format!("https://steamcommunity.com/profiles/{}/inventory/json/730/2", id)).unwrap();
    }
    
    // Convert to the steamapi struct
    let resp_json: SteamApi = serde_json::from_str::<SteamApi>(&resp.text().unwrap()).unwrap();

    // Test if we errored
    if !resp_json.success {
        // We have an api error
        println!("Couldn't get inventory");
        panic!();
    }

    // We didn't, nice
    // quick reject hashmap for errors
    // We'll try them again later
    let mut rejects: HashMap<&rgInventory, rgDescription> = HashMap::new();

    // Keep count
    let mut i = 0;

    // quickly check limit
    if limit > resp_json.rgInventory.len() {
        limit = resp_json.rgInventory.len();
    }


    // Iter through each item :cope:
    for rginv in resp_json.rgInventory.values().collect::<Vec<&rgInventory>>(){

        // See if we're past the limit
        if i > limit {
            break;
        }

        // Get the correct rgdesc
        // Hashmaps have ARBITRARY ORDERING, so we need to check every single one
        //let rgdesc = resp_json.rgDescriptions.values().collect::<Vec<&rgDescription>>()[i];
        let mut rgdesc: rgDescription = rgDescription::blank();

        for rgdesci in resp_json.rgDescriptions.values().collect::<Vec<&rgDescription>>() {
            // If the instanceids match, we have the same item
            if rgdesci.instanceid == rginv.instanceid && rgdesci.classid == rginv.classid {
                rgdesc = rgdesci.clone();
            }
        }

        if rgdesc.clone().actions.is_none() {
            println!("{:?}", rgdesc);
            continue;
        }

        // Get the link
        let mut link = rgdesc.clone().actions.unwrap()[0].link.clone();

        // Parse the link into the correct format
        // Replace %owner_steamid% with the profiles steam id
        // Replace %assetid% with the inventory item id
        link = link.replace("%owner_steamid%", id).replace("%assetid%", &rginv.id);

        //println!("{}", &link);
        println!("{} / {}, {:.3}% (Total: {}, {}%)", i, limit, (i as f32 / limit as f32) * 100 as f32, resp_json.rgInventory.len(), (i as f32 / resp_json.rgInventory.len() as f32) * 100 as f32);
        
        let mut data = String::new();

        // If we have a nametag
        if rgdesc.fraudwarnings.is_some() {
            data = get_item(&link, Some(
                rgdesc.fraudwarnings.as_ref().unwrap()[0]
                .substring(
                    rgdesc.fraudwarnings.as_ref().unwrap()[0].find("Name Tag: ''").unwrap() + 12,  // We are getting the actual name tag because we get "Name Tag: ''你不需要登顶 在山脚我也爱你''"
                    rgdesc.fraudwarnings.as_ref().unwrap()[0].rfind("''").unwrap()
                ).to_string()
                )
            );
        }

        // If we don't have one
        else {
            // Get the item's link and then data and then add that to writedata.
            data = get_item(&link, None);
        }


        // If we errored out add to rejects
        if data == "".to_string() {
            rejects.insert(rginv, rgdesc);
        }

        else {
            writedata.push_str(&data);
        }

        i += 1;

        // Spare the public api, wait half a second
        thread::sleep(time::Duration::from_millis(500));
    }

    // Try rejects again if we have -r
    if retry {
        for i in 0..rejects.len() {

        // Get the items

        let rgdesc = rejects.values().collect::<Vec<&rgDescription>>()[i];

        let rginv = rejects.keys().collect::<Vec<&&rgInventory>>()[i];

        if rgdesc.clone().actions.is_none() {
            println!("{:?}", rgdesc);
            continue;
        }

        // Get the link
        let mut link = rgdesc.clone().actions.as_ref().unwrap()[0].link.clone();

        // Parse the link into the correct format
        // Replace %owner_steamid% with the profiles steam id
        // Replace %assetid% with the inventory item id
        link = link.replace("%owner_steamid%", id).replace("%assetid%", rginv.id.as_str());

        //println!("{}", &link);
        println!("Rejects: {} / {}, {:.3}%", i, rejects.len(), (i as f32 / rejects.len() as f32) * 100 as f32);

        let mut data = String::new();

        // If we have a nametag
        if rgdesc.fraudwarnings.is_some() {
            data = get_item(&link, Some(
                rgdesc.fraudwarnings.as_ref().unwrap()[0]
                .substring(
                    rgdesc.fraudwarnings.as_ref().unwrap()[0].find("Name Tag: ''").unwrap(), // We are getting the actual name tag because we get "Name Tag: ''你不需要登顶 在山脚我也爱你''"
                    rgdesc.fraudwarnings.as_ref().unwrap()[0].rfind("''").unwrap()
                ).to_string()
                )
            );
        }

        // If we don't have one
        else {
            // Get the item's link and then data and then add that to writedata.
            data = get_item(&link, None);
        }

        // Add it to writedata
        writedata.push_str(&data);


        // Spare the public api, half a second
        thread::sleep(time::Duration::from_millis(500));
    }
    }

    // Return
    return writedata;
}