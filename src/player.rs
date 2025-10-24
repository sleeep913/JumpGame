// 导入音频相关组件，用于控制蓄力音效
use bevy::audio::AudioSink;
// 导入颜色调色板
use bevy::color::palettes;
// 导入Bevy主要组件
use bevy::prelude::*;
// 导入时间戳功能，用于蓄力计时
use bevy::utils::Instant;
// 导入粒子效果库
use bevy_hanabi::prelude::*;
// 导入数学常量
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// 导入平台相关组件
use crate::platform::PlatformShape;
// 导入UI相关组件和资源
use crate::ui::{GameSounds, GameState, ScoreUpEvent, ScoreUpQueue};
use crate::{
    platform::{CurrentPlatform, NextPlatform},
    ui::Score,
};

/// 玩家初始位置常量
/// 设置在Y=1.5，使玩家正好站在平台上（平台顶面Y=1.0）
pub const INITIAL_PLAYER_POS: Vec3 = Vec3::new(0.0, 1.5, 0.0);

/// 蓄力状态资源
/// 用于跟踪玩家跳跃前的蓄力过程
#[derive(Debug, Resource)]
pub struct Accumulator(pub Option<Instant>);

/// 蓄力音效组件
/// 标记正在播放的蓄力音效实体
#[derive(Debug, Component)]
pub struct AccumulationSound;

/// 准备跳跃计时器
/// 防止从主菜单进入游戏时立即跳跃
#[derive(Debug, Resource)]
pub struct PrepareJumpTimer(pub Timer);

/// 跳跃状态资源
/// 控制玩家跳跃动画和跳跃结果
#[derive(Debug, Resource)]
pub struct JumpState {
    /// 跳跃起始位置
    pub start_pos: Vec3,
    /// 跳跃目标位置
    pub end_pos: Vec3,
    /// 跳跃动画持续时间（秒）
    pub animation_duration: f32,
    /// 是否跳跃失败（落到平台外）
    pub falled: bool,
    /// 跳跃动画是否完成
    pub completed: bool,
}

/// 为JumpState实现默认初始化
impl Default for JumpState {
    fn default() -> Self {
        Self {
            start_pos: Vec3::ZERO,
            end_pos: Vec3::ZERO,
            animation_duration: 0.0,
            falled: false,
            completed: true,  // 初始状态为已完成，允许跳跃
        }
    }
}

impl JumpState {
    /// 开始跳跃动画
    /// 
    /// # 参数
    /// - `start_pos`: 跳跃起始位置
    /// - `end_pos`: 跳跃目标位置
    /// - `animation_duration`: 动画持续时间
    pub fn animate_jump(&mut self, start_pos: Vec3, end_pos: Vec3, animation_duration: f32) {
        info!("Start jump!");
        self.start_pos = start_pos;
        self.end_pos = end_pos;
        self.animation_duration = animation_duration;
        self.completed = false;  // 标记动画开始，未完成
    }
}

/// 摔落状态资源
/// 控制玩家摔落动画和类型
#[derive(Debug, Resource)]
pub struct FallState {
    /// 摔落起始位置
    pub pos: Vec3,
    /// 摔落类型（笔直或倾斜）
    pub fall_type: FallType,
    /// 是否完成倾斜动作（仅在倾斜摔落时使用）
    pub tilt_completed: bool,
    /// 摔落动画是否完全完成
    pub completed: bool,
    /// 是否已经播放摔落音效
    pub played_sound: bool,
}

/// 摔落类型枚举
#[derive(Debug)]
pub enum FallType {
    /// 笔直下落
    Straight,
    /// 先倾斜再下落，Vec3代表倾斜方向
    Tilt(Vec3),
}

/// 为FallState实现默认初始化
impl Default for FallState {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            fall_type: FallType::Straight,
            tilt_completed: true,
            completed: true,  // 初始状态为已完成，不影响游戏逻辑
            played_sound: true,  // 初始状态为已播放，避免误触发音效
        }
    }
}

impl FallState {
    /// 开始笔直摔落动画
    /// 
    /// # 参数
    /// - `pos`: 摔落起始位置
    pub fn animate_straight_fall(&mut self, pos: Vec3) {
        info!("Start straight fall!");
        self.pos = pos;
        self.fall_type = FallType::Straight;
        self.completed = false;  // 标记动画开始
        self.played_sound = false;  // 重置音效播放状态
    }
    
    /// 开始倾斜摔落动画（碰到平台边缘时使用）
    /// 
    /// # 参数
    /// - `pos`: 摔落起始位置
    /// - `direction`: 倾斜方向向量
    pub fn animate_tilt_fall(&mut self, pos: Vec3, direction: Vec3) {
        info!("Start tilt fall!");
        self.pos = pos;
        self.fall_type = FallType::Tilt(direction);
        self.tilt_completed = false;  // 倾斜动作未完成
        self.completed = false;  // 整体动画未完成
        self.played_sound = false;  // 重置音效播放状态
    }
}

/// 玩家标记组件
/// 用于标识和查询玩家实体
#[derive(Debug, Component)]
pub struct Player;

/// 蓄力粒子效果生成计时器
/// 控制蓄力时粒子效果的生成频率
#[derive(Debug, Resource)]
pub struct GenerateAccumulationParticleEffectTimer(pub Timer);

/// 设置玩家实体
/// 
/// 创建玩家角色（一个粉色胶囊体）并播放游戏开始音效
pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_sounds: Res<GameSounds>,
) {
    // 创建玩家实体：一个粉色胶囊体
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.2, 0.5).mesh())),  // 胶囊体：半径0.2，高度0.5
        MeshMaterial3d(materials.add(Color::Srgba(palettes::css::PINK))),  // 粉色材质
        Transform::from_translation(INITIAL_PLAYER_POS),  // 设置初始位置
        Player,  // 添加玩家标记组件
    ));
    
    // 播放游戏开始音效
    commands.spawn((
        AudioPlayer(game_sounds.start.clone()),  // 使用开始游戏音效
        PlaybackSettings::DESPAWN,  // 播放完成后自动销毁
    ));
}

