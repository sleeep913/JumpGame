// 导入音频处理相关功能
use bevy::audio::AudioSink;
// 导入颜色调色板
use bevy::color::palettes;
// 导入Bevy核心组件和功能
use bevy::prelude::*;
// 导入时间戳功能，用于计算蓄力时长
use bevy::utils::Instant;
// 导入粒子效果库
use bevy_hanabi::prelude::*;
// 导入数学常量，用于旋转计算
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// 导入平台相关组件
use crate::platform::PlatformShape;
// 导入UI和游戏状态相关组件
use crate::ui::{GameSounds, GameState, ScoreUpEvent, ScoreUpQueue};
// 导入平台标记组件和分数组件
use crate::{
    platform::{CurrentPlatform, NextPlatform},
    ui::Score,
};

/// 玩家初始位置常量
pub const INITIAL_PLAYER_POS: Vec3 = Vec3::new(0.0, 1.5, 0.0);

/// 蓄力资源，存储蓄力开始的时间戳
#[derive(Debug, Resource)]
pub struct Accumulator(pub Option<Instant>);

/// 蓄力音效组件标记
#[derive(Debug, Component)]
pub struct AccumulationSound;

/// 准备跳跃计时器，防止从主菜单进入游戏时立即跳跃
#[derive(Debug, Resource)]
pub struct PrepareJumpTimer(pub Timer);

/// 跳跃状态资源，管理跳跃动画和逻辑
#[derive(Debug, Resource)]
pub struct JumpState {
    pub start_pos: Vec3,       // 跳跃起始位置
    pub end_pos: Vec3,         // 跳跃目标位置
    pub animation_duration: f32, // 跳跃动画时长，秒
    pub falled: bool,          // 是否摔落
    pub completed: bool,       // 跳跃是否完成
}
/// JumpState的默认实现
impl Default for JumpState {
    fn default() -> Self {
        Self {
            start_pos: Vec3::ZERO,
            end_pos: Vec3::ZERO,
            animation_duration: 0.0,
            falled: false,
            completed: true, // 默认初始状态为已完成
        }
    }
}

/// JumpState的方法实现
impl JumpState {
    /// 初始化跳跃动画状态
    /// 
    /// # 参数
    /// - `start_pos`: 跳跃起始位置
    /// - `end_pos`: 跳跃结束位置
    /// - `animation_duration`: 跳跃动画持续时间
    pub fn animate_jump(&mut self, start_pos: Vec3, end_pos: Vec3, animation_duration: f32) {
        info!("Start jump!");
        self.start_pos = start_pos;
        self.end_pos = end_pos;
        self.animation_duration = animation_duration;
        self.completed = false; // 标记为跳跃中
    }
}

/// 摔落状态资源，管理角色摔落动画和逻辑
#[derive(Debug, Resource)]
pub struct FallState {
    pub pos: Vec3,            // 摔落起始位置
    pub fall_type: FallType,  // 摔落类型
    pub tilt_completed: bool, // 是否完成倾斜动作
    pub completed: bool,      // 是否所有动作完成
    pub played_sound: bool,   // 是否已播放摔落音效
}

/// 摔落类型枚举
#[derive(Debug)]
pub enum FallType {
    Straight,           // 笔直下落
    Tilt(Vec3),         // 先倾斜再下落，Vec3代表倾斜方向
}
/// FallState的默认实现
impl Default for FallState {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            fall_type: FallType::Straight,
            tilt_completed: true,     // 默认已完成倾斜
            completed: true,          // 默认已完成
            played_sound: true,       // 默认已播放音效
        }
    }
}

/// FallState的方法实现
impl FallState {
    /// 初始化笔直下落动画
    /// 
    /// # 参数
    /// - `pos`: 摔落起始位置
    pub fn animate_straight_fall(&mut self, pos: Vec3) {
        info!("Start straight fall!");
        self.pos = pos;
        self.fall_type = FallType::Straight;
        self.completed = false;
        self.played_sound = false;
    }
    
    /// 初始化倾斜后下落动画
    /// 
    /// # 参数
    /// - `pos`: 摔落起始位置
    /// - `direction`: 倾斜方向
    pub fn animate_tilt_fall(&mut self, pos: Vec3, direction: Vec3) {
        info!("Start tilt fall!");
        self.pos = pos;
        self.fall_type = FallType::Tilt(direction);
        self.tilt_completed = false; // 标记倾斜未完成
        self.completed = false;
        self.played_sound = false;
    }
}

/// 玩家组件标记，用于查询玩家实体
#[derive(Debug, Component)]
pub struct Player;

/// 蓄力粒子效果生成计时器
#[derive(Debug, Resource)]
pub struct GenerateAccumulationParticleEffectTimer(pub Timer);

/// 设置玩家实体
/// 
/// 创建玩家角色模型并播放开始音效
/// 
/// # 参数
/// - `commands`: 命令系统，用于创建实体
/// - `meshes`: 网格资源管理器
/// - `materials`: 材质资源管理器
/// - `game_sounds`: 游戏音效资源
pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_sounds: Res<GameSounds>,
) {
    // 创建玩家实体，使用胶囊体模型，粉色材质
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.2, 0.5).mesh())), // 添加胶囊体网格，半径0.2，高度0.5
        MeshMaterial3d(materials.add(Color::Srgba(palettes::css::PINK))), // 添加粉色材质
        Transform::from_translation(INITIAL_PLAYER_POS), // 设置初始位置
        Player, // 添加玩家组件标记
    ));
    // 播放游戏开始音效
    commands.spawn((
        AudioPlayer(game_sounds.start.clone()), // 开始音效
        PlaybackSettings::DESPAWN, // 播放结束后自动销毁
    ));
}

/// 玩家跳跃逻辑系统
/// 
/// 处理鼠标输入、蓄力计算、跳跃轨迹计算和平台检测
pub fn player_jump(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mut score: ResMut<Score>,
    mut accumulator: ResMut<Accumulator>,
    mut jump_state: ResMut<JumpState>,
    mut fall_state: ResMut<FallState>,
    mut score_up_queue: ResMut<ScoreUpQueue>,
    prepare_jump_timer: Res<PrepareJumpTimer>,
    time: Res<Time<Real>>,
    game_sounds: Res<GameSounds>,
    q_accumulation_sound: Query<&AudioSink, With<AccumulationSound>>,
    q_player: Query<&Transform, With<Player>>,
    q_current_platform: Query<(Entity, &Transform, &PlatformShape), With<CurrentPlatform>>,
    q_next_platform: Query<(Entity, &Transform, &PlatformShape), With<NextPlatform>>,
) {
    // 检查准备跳跃计时器是否完成
    // 如果未完成，说明刚从主菜单进入游戏，忽略输入
    if !prepare_jump_timer.0.finished() {
        return;
    }
    
    // 鼠标左键按下，开始蓄力
    // 只有当前跳跃和摔落都已完成时才响应
    if buttons.just_pressed(MouseButton::Left) && jump_state.completed && fall_state.completed {
        // 记录蓄力开始时间
        accumulator.0 = time.last_update();
        // 播放蓄力音效（循环播放）
        commands.spawn((
            AccumulationSound, // 标记为蓄力音效
            AudioPlayer(game_sounds.accumulation.clone()), // 蓄力音效资源
            PlaybackSettings::LOOP, // 循环播放设置
        ));
    }
    
    // 鼠标左键释放，结束蓄力并执行跳跃
    // 检查条件：跳跃完成、摔落完成、正在蓄力中
    if buttons.just_released(MouseButton::Left)
        && jump_state.completed
        && fall_state.completed
        && accumulator.0.is_some()
    {
        // 检查是否存在下一个平台，不存在则无法跳跃
        if q_next_platform.is_empty() {
            warn!("There is no next platform");
            return;
        }
        // 获取当前平台、下一个平台和玩家的信息
        let (current_platform_entity, current_platform_transform, current_platform_shape) =
            q_current_platform.single();
        let (next_platform_entity, next_platform_transform, next_platform_shape) =
            q_next_platform.single();
        let player = q_player.single();

        // 计算跳跃后的落点位置
        // 根据平台排列方向(X轴或Z轴)决定跳跃方向
        let landing_pos = if (next_platform_transform.translation.x
            - current_platform_transform.translation.x)
            < 0.1  // 如果X轴差值小于0.1，说明平台排列在Z轴方向
        {
            // Z轴方向跳跃计算
            Vec3::new(
                player.translation.x,  // X轴位置不变
                INITIAL_PLAYER_POS.y,  // Y轴高度保持初始位置
                player.translation.z
                    - 3.0 * accumulator.0.as_ref().unwrap().elapsed().as_secs_f32(), // Z轴位移与蓄力时间成正比
            )
        } else {  // 否则平台排列在X轴方向
            // X轴方向跳跃计算
            Vec3::new(
                player.translation.x
                    + 3.0 * accumulator.0.as_ref().unwrap().elapsed().as_secs_f32(), // X轴位移与蓄力时间成正比
                INITIAL_PLAYER_POS.y,  // Y轴高度保持初始位置
                player.translation.z,  // Z轴位置不变
            )
        };
        
        // 调试信息输出
        dbg!(player.translation);
        dbg!(accumulator.0.as_ref().unwrap().elapsed().as_secs_f32());

        // 初始化跳跃动画
        // 跳跃持续时间与蓄力时长成正比，但至少为0.5秒
        jump_state.animate_jump(
            player.translation,      // 起始位置
            landing_pos,             // 目标位置
            (accumulator.0.as_ref().unwrap().elapsed().as_secs_f32() / 2.0).max(0.5), // 动画持续时间
        );

        // 平台检测：判断角色是否落在平台上
        // 检查条件：要么落在当前平台，要么落在下一个平台
        if current_platform_shape
            .is_landed_on_platform(current_platform_transform.translation, landing_pos)
            || next_platform_shape
                .is_landed_on_platform(next_platform_transform.translation, landing_pos)
        {
            // 成功跳跃，未摔落
            jump_state.falled = false;
            
            // 如果落在了下一个平台上
            if next_platform_shape
                .is_landed_on_platform(next_platform_transform.translation, landing_pos)
            {
                // 分数加1
                score.0 += 1;
                
                // 添加分数上升动画事件
                score_up_queue.0.push(ScoreUpEvent {
                    landing_pos: Vec3::new(landing_pos.x, landing_pos.y + 0.5, landing_pos.z),
                });

                // 更新平台状态：
                // 1. 移除下一个平台的NextPlatform标记
                commands.entity(next_platform_entity).remove::<NextPlatform>();
                // 2. 为下一个平台添加CurrentPlatform标记
                commands.entity(next_platform_entity).insert(CurrentPlatform);
                // 3. 移除当前平台的CurrentPlatform标记
                commands.entity(current_platform_entity).remove::<CurrentPlatform>();
            }

        // 蓄力不足或蓄力过度，角色摔落
        } else {
            // 标记为摔落状态
            jump_state.falled = true;
            
            // 根据碰撞情况决定摔落类型
            // 1. 是否碰到当前平台边缘
            if current_platform_shape.is_touched_player(
                current_platform_transform.translation,
                landing_pos,
                0.2,  // 接触检测半径
            ) {
                info!("Player touched current platform");
                // 根据跳跃方向确定倾斜方向
                let fall_direction = if landing_pos.x == player.translation.x {
                    Vec3::NEG_X
                } else {
                    Vec3::NEG_Z
                };
                // 初始化倾斜摔落动画
                fall_state.animate_tilt_fall(landing_pos, fall_direction);
            }
            // 2. 是否碰到下一个平台边缘
            else if next_platform_shape.is_touched_player(
                next_platform_transform.translation,
                landing_pos,
                0.2,
            ) {
                info!("Player touched next platform");
                // 根据跳跃方向和位置确定倾斜方向
                let fall_direction = if landing_pos.x == player.translation.x {
                    if landing_pos.z < next_platform_transform.translation.z {
                        Vec3::NEG_X
                    } else {
                        Vec3::X
                    }
                } else {
                    if landing_pos.x < next_platform_transform.translation.x {
                        Vec3::Z
                    } else {
                        Vec3::NEG_Z
                    }
                };
                // 初始化倾斜摔落动画
                fall_state.animate_tilt_fall(landing_pos, fall_direction);
            }
            // 3. 完全没碰到平台，直接下落
            else {
                fall_state.animate_straight_fall(landing_pos);
            }
        }

        // 结束蓄力状态
        accumulator.0 = None;
        
        // 停止蓄力音效
        for sink in q_accumulation_sound.iter() {
            sink.pause();
        }
    }
}

/// 跳跃动画系统
/// 
/// 实现玩家跳跃的弧形轨迹和旋转动画
pub fn animate_jump(
    mut commands: Commands,
    mut jump_state: ResMut<JumpState>,
    time: Res<Time>,
    mut q_player: Query<&mut Transform, With<Player>>,
    game_sounds: Res<GameSounds>,
) {
    // 只有当跳跃未完成时执行动画
    if !jump_state.completed {
        let mut player = q_player.single_mut();

        // 计算跳跃轨迹的中心点（用于圆周运动）
        let around_point = Vec3::new(
            (jump_state.start_pos.x + jump_state.end_pos.x) / 2.0, // 中心点X坐标
            (jump_state.start_pos.y + jump_state.end_pos.y) / 2.0, // 中心点Y坐标
            (jump_state.start_pos.z + jump_state.end_pos.z) / 2.0, // 中心点Z坐标
        );

        // 确定旋转轴：根据跳跃方向确定
        let rotate_axis = if (jump_state.end_pos.x - jump_state.start_pos.x) < 0.1 {
            Vec3::X  // Z轴方向跳跃，绕X轴旋转
        } else {
            Vec3::Z  // X轴方向跳跃，绕Z轴旋转
        };
        
        // 计算旋转四元数
        // 旋转速度与动画持续时间成反比，确保在指定时间内完成180度旋转
        let quat = Quat::from_axis_angle(
            rotate_axis,
            -(1.0 / jump_state.animation_duration) * PI * time.delta_secs(),
        );

        // 预测下一帧位置，用于判断是否到达跳跃底部
        let mut clone_player = player.clone();
        clone_player.translate_around(around_point, quat);
        
        // 判断是否到达跳跃底部
        if clone_player.translation.y < INITIAL_PLAYER_POS.y {
            // 到达目标位置，结束跳跃
            player.translation = jump_state.end_pos;
            player.rotation = Quat::IDENTITY; // 重置旋转

            // 标记跳跃完成
            jump_state.completed = true;
            
            // 如果成功跳跃（未摔落），播放成功音效
            if !jump_state.falled {
                commands.spawn((
                    AudioPlayer(game_sounds.success.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        } else {
            // 继续执行跳跃动画
            player.translate_around(around_point, quat);

            // 角色自身旋转动画
            player.rotate_local_axis(
                Dir3::new_unchecked(rotate_axis),
                -(1.0 / jump_state.animation_duration) * TAU * time.delta_secs(), // 完成360度旋转
            );
        }
    }
}

// 角色蓄力效果
// TODO 蓄力过程中保持与平台相接触
pub fn animate_player_accumulation(
    accumulator: Res<Accumulator>,
    mut q_player: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut player = q_player.single_mut();
    match accumulator.0 {
        Some(_) => {
            player.scale.x = (player.scale.x + 0.12 * time.delta_secs()).min(1.3);
            player.scale.y = (player.scale.y - 0.15 * time.delta_secs()).max(0.6);
            player.scale.z = (player.scale.z + 0.12 * time.delta_secs()).min(1.3);
        }
        None => {
            player.scale = Vec3::ONE;
        }
    }
}

/// 摔落动画系统
/// 
/// 处理玩家摔落时的动画效果，包括笔直下落和倾斜后下落两种类型
/// 
/// # 参数
/// - `commands`: 命令系统，用于播放音效
/// - `fall_state`: 摔落状态资源，控制摔落动画的进程
/// - `jump_state`: 跳跃状态资源，确保跳跃完成后才开始摔落
/// - `time`: 时间资源，控制动画速度
/// - `next_game_state`: 游戏状态资源，在摔落后切换到游戏结束状态
/// - `q_player`: 玩家实体查询，更新玩家位置和旋转
/// - `game_sounds`: 游戏音效资源，播放摔落音效
pub fn animate_fall(
    mut commands: Commands,
    mut fall_state: ResMut<FallState>,
    jump_state: Res<JumpState>,
    time: Res<Time>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut q_player: Query<&mut Transform, With<Player>>,
    game_sounds: Res<GameSounds>,
) {
    // 只有当摔落未完成且跳跃已完成时执行摔落动画
    if !fall_state.completed && jump_state.completed {
        // 播放摔落音效（仅播放一次）
        if !fall_state.played_sound {
            commands.spawn((
                AudioPlayer(game_sounds.fall.clone()),
                PlaybackSettings::DESPAWN,
            ));
            fall_state.played_sound = true;
        }
        
        // 获取玩家实体
        let mut player = q_player.single_mut();
        
        // 根据摔落类型执行不同的动画逻辑
        match fall_state.fall_type {
            // 笔直下落类型
            FallType::Straight => {
                // 判断是否摔落到底部
                if player.translation.y < 0.5 {
                    // 标记摔落完成
                    fall_state.completed = true;
                    info!("Game over!");
                    // 切换到游戏结束状态
                    next_game_state.set(GameState::GameOver);
                } else {
                    // 持续向下移动（速度为0.7单位/秒）
                    player.translation.y -= 0.7 * time.delta_secs();
                }
            }
            
            // 倾斜后下落类型
            FallType::Tilt(direction) => {
                if !fall_state.tilt_completed {
                    // 第一阶段：倾斜动作
                    // 设置旋转中心点（略低于初始位置）
                    let around_point = Vec3::new(
                        fall_state.pos.x,
                        INITIAL_PLAYER_POS.y - 0.5,
                        fall_state.pos.z,
                    );
                    
                    // 检查是否完成倾斜
                    if player.translation.y < around_point.y {
                        fall_state.tilt_completed = true;
                    } else {
                        // 计算旋转四元数（每秒旋转90度）
                        let quat = 
                            Quat::from_axis_angle(direction, 1.0 * FRAC_PI_2 * time.delta_secs());
                        // 围绕指定点旋转玩家
                        player.rotate_around(around_point, quat);
                    }
                } else {
                    // 第二阶段：下坠动作
                    // 判断是否摔落到底部
                    if player.translation.y < 0.2 {
                        // 标记摔落完成
                        fall_state.completed = true;
                        info!("Game over!");
                        // 切换到游戏结束状态
                        next_game_state.set(GameState::GameOver);
                    } else {
                        // 持续向下移动（速度为0.7单位/秒）
                        player.translation.y -= 0.7 * time.delta_secs();
                    }
                }
            }
        }
    }
}

/// 蓄力粒子效果生成系统
/// 
/// 在玩家蓄力过程中生成粒子效果，提供视觉反馈，粒子从红渐变到黄再到白
/// 
/// # 参数
/// - `commands`: 命令系统，用于生成粒子效果实体
/// - `effects`: 粒子效果资源管理器
/// - `accumulator`: 蓄力状态资源，判断是否处于蓄力状态
/// - `effect_timer`: 粒子效果生成计时器，控制生成频率
/// - `time`: 时间资源
/// - `q_effect`: 粒子效果查询，用于在蓄力结束时清理粒子
/// - `q_player`: 玩家实体查询，获取玩家位置
pub fn animate_accumulation_particle_effect(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    accumulator: Res<Accumulator>,
    mut effect_timer: ResMut<GenerateAccumulationParticleEffectTimer>,
    time: Res<Time>,
    mut q_effect: Query<(Entity, &mut ParticleEffect, &mut Transform)>,
    q_player: Query<&Transform, (With<Player>, Without<ParticleEffect>)>,
) {
    // 当玩家正在蓄力时生成粒子效果
    if accumulator.0.is_some() {
        // 计时器控制粒子生成频率
        effect_timer.0.tick(time.delta());
        if effect_timer.0.just_finished() {
            // 获取玩家位置
            let player = q_player.single();
            
            // 定义粒子颜色渐变（由白渐变到黄再到红，最后消失）
            let mut color_gradient = Gradient::new();
            color_gradient.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0)); // 白色（过亮）
            color_gradient.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0)); // 黄色
            color_gradient.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0)); // 红色
            color_gradient.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0)); // 完全透明

            // 定义粒子大小渐变（保持初始大小一段时间后消失）
            let mut size_gradient = Gradient::new();
            size_gradient.add_key(0.0, Vec3::splat(0.05)); // 初始大小
            size_gradient.add_key(0.3, Vec3::splat(0.05)); // 保持大小
            size_gradient.add_key(1.0, Vec3::splat(0.0));  // 消失

            // 为粒子效果实体创建唯一名称
            let name = format!("accumulation{}", time.elapsed_secs() as u32);
            
            // 创建粒子效果模块
            let mut module = Module::default();

            // 设置粒子初始位置（球形区域内）
            let init_pos = SetPositionSphereModifier {
                center: module.lit(player.translation), // 中心点在玩家位置
                radius: module.lit(1.0),                // 半径1.0
                dimension: ShapeDimension::Volume,      // 体积维度
            };

            // 设置粒子生命周期（2秒）
            let lifetime = module.lit(2.);
            let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

            // 设置粒子线性阻力（8.0）
            let update_linear_drag = LinearDragModifier::constant(&mut module, 8.0);

            // 创建并配置粒子效果资源
            let effect = effects.add(
                EffectAsset::new(3, Spawner::once(3.0.into(), true), module)
                    .init(init_pos)                    // 初始化位置
                    .init(init_lifetime)                // 初始化生命周期
                    .update(update_linear_drag)         // 更新线性阻力
                    // .update(update_force_field)       // 注释掉的力场效果
                    .render(ColorOverLifetimeModifier { // 颜色随时间变化
                        gradient: color_gradient.clone(),
                    })
                    .render(SizeOverLifetimeModifier {  // 大小随时间变化
                        gradient: size_gradient.clone(),
                        screen_space_size: false,       // 使用世界空间大小
                    }),
            );
            
            // 生成粒子效果实体
            commands.spawn((
                Name::new(name),                      // 设置实体名称
                ParticleEffectBundle {
                    effect: ParticleEffect::new(effect), // 设置粒子效果
                    transform: Transform::IDENTITY,      // 设置变换
                    ..Default::default()
                },
            ));

            // 重置计时器
            effect_timer.0.reset();
        }
    } else {
        // 当蓄力结束时，清理所有粒子效果
        for (entity, _, _) in &mut q_effect {
            commands.entity(entity).despawn();
        }
    }
}

/// 清理玩家实体系统
/// 
/// 在游戏结束或重置时销毁玩家实体
/// 
/// # 参数
/// - `commands`: 命令系统，用于销毁实体
/// - `q_player`: 玩家实体查询
pub fn clear_player(mut commands: Commands, q_player: Query<Entity, With<Player>>) {
    for player in &q_player {
        commands.entity(player).despawn();
    }
}

/// 准备跳跃计时器更新系统
/// 
/// 更新准备跳跃计时器，防止从主菜单进入游戏时立即跳跃
/// 
/// # 参数
/// - `time`: 时间资源
/// - `prepare_timer`: 准备跳跃计时器资源
pub fn prepare_jump(time: Res<Time>, mut prepare_timer: ResMut<PrepareJumpTimer>) {
    prepare_timer.0.tick(time.delta());
}

/// 重置准备跳跃计时器系统
/// 
/// 在需要时重置准备跳跃计时器，通常在游戏状态切换时使用
/// 
/// # 参数
/// - `prepare_timer`: 准备跳跃计时器资源
pub fn reset_prepare_jump_timer(mut prepare_timer: ResMut<PrepareJumpTimer>) {
    prepare_timer.0.reset();
}
