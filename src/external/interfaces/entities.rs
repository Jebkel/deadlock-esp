use std::ffi::c_void;

use egui::{Pos2, Rect};

use crate::{external::{cheat::esp::*, offsets::client_dll::CBasePlayerController}, memory::read_memory, settings::structs::Settings};
use super::{math::Matrix, structs::{Controller, GameSceneNode, Pawn, PlayerDataGlobal, Skeleton}};

trait EntityBase
{
    fn read_base(&mut self, index: i32, entity_list_ptr: *mut c_void) -> *mut c_void {
        unsafe {
            read_memory(entity_list_ptr.add((((index & 0x7FFF) >> 9) * 8 + 16) as usize))
        }
    }
}

pub struct Player
{
    pub index: i32,
    pub pawn: Pawn,
    pub rect: Rect,

    pub game_scene_node: GameSceneNode,
    pub controller: Controller,

    pub skeleton: Skeleton,
    pub data: PlayerDataGlobal
}

impl EntityBase for Player
{
}

impl Player
{
    pub fn new(index: i32) -> Self
    {
        Self {
            index,
            game_scene_node: Default::default(),
            controller: Default::default(),
            pawn: Pawn::default(),
            skeleton: Skeleton::default(),
            data: PlayerDataGlobal::default(),
            rect: Rect { min: Pos2::default(), max: Pos2::default() }
        }
    }
    
    pub fn is_invalid(&self) -> bool
    {
        self.controller.ptr as usize == 0
    }

    pub fn is_alive(&self) -> bool
    {
        self.data.alive && self.pawn.health > 0
    }

    pub fn dead(&mut self)
    {
        self.data.alive = false;    
        self.rect = Rect {
            min: Default::default(),
            max: Default::default(),
        }
    }

    pub fn update(&mut self, entity_list_ptr: *mut c_void,  matrix: &Matrix) -> bool {
        let base_ptr = self.read_base(self.index, entity_list_ptr);
        if base_ptr as usize != 0
        {
            self.controller.update(base_ptr, self.index);
            unsafe {
                if !self.is_invalid()
                {
                    let pawn_handle: *mut c_void = read_memory(self.controller.ptr.add(CBasePlayerController::m_hPawn));
                    let list_entry: *mut c_void = read_memory(entity_list_ptr.add(0x8 * ((pawn_handle as usize & 0x7FFF) >> 0x9) + 0x10));
                    self.pawn.update(list_entry, pawn_handle);
                    if self.pawn.ptr as i32 != 0
                    {
                        self.game_scene_node.update(self.pawn.ptr);
                        self.data.update(self.controller.ptr);
                        self.skeleton.update(self.pawn.ptr, self.data.hero);
                        self.update_rect(matrix);
                    }
                    else
                    {
                        self.dead();
                    }
                }
                else
                {
                    self.dead();
                }
            }
        }
        self.controller.local
    }

    fn update_rect(&mut self, matrix: &Matrix)
    {
        let mut point_top = self.skeleton.head_pos.clone();
        point_top.z += 10.;
        if matrix.transform(&mut point_top)
        {
            let mut point_bottom = self.game_scene_node.position.clone();
            point_bottom.z -= 10.;
            matrix.transform(&mut point_bottom);

            let height: f32 = point_bottom.y - point_top.y;
            let width: f32 = height / 2. * 1.15;
            self.rect = Rect::from_pos(Pos2 { x: point_bottom.x - (width / 2.), y: point_top.y });
            self.rect.set_width(width);
            self.rect.set_height(height);
        }
        else
        {
            self.rect = Rect {
                min: Default::default(),
                max: Default::default(),
            }
        }
    }

    pub fn draw(&self, matrix: &Matrix, g: &egui::Painter, settings: &Settings)
    {
        let mut screen_pos = self.game_scene_node.position.clone();
        if matrix.transform(&mut screen_pos)
        {
            boxes::draw_boxes(self.rect, g, settings);
            text::draw(g, self, settings);
        }
    }
}