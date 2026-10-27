use crate::data::{MasteryPerkId,MAX_SKILL_LEVEL};
use crate::state::GameState;
use crate::ui::Tui;
use rand::Rng;
use std::io;
pub fn has_perk(s:&GameState,p:MasteryPerkId)->bool{s.mastery_perks.contains(&p)}
pub fn unlocked_mastery_slots(s:&GameState)->usize{
 let lv=[s.player.skills.woodcutting.level,s.player.skills.cooking.level,s.player.skills.fishing.level,s.player.skills.mining.level,s.player.skills.smithing.level,s.player.skills.thieving.level];
 let mut slots=if lv.iter().any(|n|*n>=MAX_SKILL_LEVEL){1}else{0};let total=s.player.skills.total_level();
 if total>=300{slots=slots.max(2)}if total>=400{slots=slots.max(3)}if total>=500{slots=slots.max(4)}slots.min(MasteryPerkId::ALL.len())
}
pub fn critical_chance_pct(s:&GameState)->u32{(5+if has_perk(s,MasteryPerkId::LethalPrecision){10}else{0}).min(25)}
pub fn vendor_discount_pct(s:&GameState)->u32{if has_perk(s,MasteryPerkId::SilverTongue){15}else{0}}
pub fn vendor_price(s:&GameState,base:u32,extra:u32)->u32{let d=vendor_discount_pct(s).saturating_add(extra).min(30);(u64::from(base)*u64::from(100-d)/100).min(u64::from(u32::MAX)) as u32}
pub fn discounted_ingredient_count(n:u32)->u32{(u64::from(n)*85/100).max(1).min(u64::from(u32::MAX)) as u32}
pub fn gathering_yield_at(base:u32,enabled:bool,roll:u32)->u32{if !enabled||base==0{return base}base.saturating_add(base/4).saturating_add(if roll%100<(base%4)*25 {1}else{0})}
pub fn gathering_yield(s:&GameState,base:u32)->u32{gathering_yield_at(base,has_perk(s,MasteryPerkId::LumberjacksGreed),rand::thread_rng().gen_range(0..100))}
pub fn mastery_menu(tui:&mut Tui,s:&mut GameState)->io::Result<()>{
 loop{let slots=unlocked_mastery_slots(s);let mut labels:Vec<String>=MasteryPerkId::ALL.iter().map(|p|format!("{} {} — {}",if has_perk(s,*p){"[Equipped]"}else{"[Available]"},p.name(),p.description())).collect();labels.push("Back".into());let refs:Vec<&str>=labels.iter().map(String::as_str).collect();let n=tui.select_menu(&format!("Mastery Perks · {} / {} slots",s.mastery_perks.len(),slots),&refs,Some(s))?;if n>=MasteryPerkId::ALL.len(){return Ok(())}let p=MasteryPerkId::ALL[n];
 if let Some(i)=s.mastery_perks.iter().position(|x|*x==p){s.mastery_perks.remove(i);tui.show_message("Perk removed",&format!("{} is no longer active.",p.name()),Some(s))?}
 else if slots==0{tui.show_message("Mastery locked","Reach level 100 in one skill to unlock a slot; total skill levels 300/400/500 unlock up to four slots.",Some(s))?}
 else if s.mastery_perks.len()>=slots{tui.show_message("No free slots","Unequip a perk or reach the next total-level milestone.",Some(s))?}
 else{s.mastery_perks.push(p);tui.show_message("Perk equipped",&format!("{} is now active.",p.name()),Some(s))?}
 }
}
#[cfg(test)]mod tests{use super::*;use crate::state::Player;fn state()->GameState{GameState::new(Player::new("Hero".into(),String::new()))}
#[test]fn mastery_slots_unlock_on_skill_milestones(){let mut s=state();assert_eq!(unlocked_mastery_slots(&s),0);s.player.skills.woodcutting.level=100;assert_eq!(unlocked_mastery_slots(&s),1);s.player.skills.cooking.level=100;s.player.skills.fishing.level=100;assert_eq!(unlocked_mastery_slots(&s),2);s.player.skills.mining.level=100;assert_eq!(unlocked_mastery_slots(&s),3);s.player.skills.smithing.level=100;assert_eq!(unlocked_mastery_slots(&s),4);}
#[test]fn vendor_discounts_share_a_thirty_percent_cap(){let mut s=state();s.mastery_perks.push(MasteryPerkId::SilverTongue);assert_eq!(vendor_price(&s,1000,0),850);assert_eq!(vendor_price(&s,1000,25),700);}
#[test]fn ingredient_discount_keeps_at_least_one(){assert_eq!(discounted_ingredient_count(1),1);assert_eq!(discounted_ingredient_count(2),1);assert_eq!(discounted_ingredient_count(10),8);}
#[test]fn gathering_bonus_averages_twenty_five_percent(){assert_eq!(gathering_yield_at(100,true,99),125);assert_eq!(gathering_yield_at(3,true,0),4);assert_eq!(gathering_yield_at(3,true,99),3);assert_eq!(gathering_yield_at(u32::MAX,true,0),u32::MAX);}
}