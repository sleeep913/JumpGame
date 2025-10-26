// 引入时间相关功能，用于设置游戏中的计时器
use std::time::Duration;

// 导入游戏各模块中的所有公共功能
use crate::camera::*;    // 相机相关功能
use crate::platform::*;  // 平台相关功能
use crate::player::*;    // 玩家相关功能
use crate::ui::*;        // UI和游戏状态相关功能

// 导入Bevy游戏引擎的主要功能
use bevy::prelude::*;
// 导入粒子效果插件（用于蓄力特效）
use bevy_hanabi::prelude::*;

// 声明游戏的各个模块
mod camera;    // 处理相机设置和跟随
mod platform;  // 处理平台生成和逻辑
mod player;    // 处理玩家角色的行为和动画
mod ui;        // 处理用户界面和游戏状态

/// 游戏的主入口函数
/// 
/// 在这里设置游戏的所有系统、资源和状态管理流程
/// 游戏使用Bevy的状态机模式管理不同的游戏阶段（主菜单、游戏进行、游戏结束）
fn main() {
    // 创建新的Bevy应用实例
    let mut app = App::new();
    
    // 添加Bevy的默认插件（渲染、窗口管理、输入处理等核心功能）
    app.add_plugins(DefaultPlugins);

    // 仅在非Web平台添加粒子效果插件
    // Web平台(wasm32)可能不支持某些粒子效果功能
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(HanabiPlugin);
    }

    // 初始化游戏状态和各种资源
    // 这些资源将在整个游戏运行过程中保持，并可被不同系统访问和修改
    app
        // 初始化游戏状态机，默认为主菜单状态
        .init_state::<GameState>()
        
        // 相机移动状态资源，用于控制相机跟随逻辑
        .insert_resource(CameraMoveState::default())
        
        // 游戏分数资源，初始为0
        .insert_resource(Score(0))
        
        // 蓄力状态资源，存储玩家当前的蓄力值和开始时间
        .insert_resource(Accumulator(None))
        
        // 跳跃状态资源，控制跳跃动画和逻辑流程
        .insert_resource(JumpState::default())
        
        // 摔落状态资源，控制摔落动画和逻辑流程
        .insert_resource(FallState::default())
        
        // 蓄力粒子特效计时器，控制特效生成频率（每200毫秒生成一次）
        .insert_resource(GenerateAccumulationParticleEffectTimer(Timer::new(
            Duration::from_millis(200),
            TimerMode::Once, // 一次性计时器，每次触发后需手动重置
        )))
        
        // 准备跳跃计时器，防止从主菜单进入游戏时立即响应输入
        // 提供一个小的缓冲时间，改善游戏体验
        .insert_resource(PrepareJumpTimer(Timer::new(
            Duration::from_millis(200),
            TimerMode::Once,
        )))
        
        // 分数上升效果队列，用于存储和显示得分动画信息
        .insert_resource(ScoreUpQueue(Vec::new()))
        
        // ===== 启动时执行的系统 =====
        // 这些系统仅在游戏首次启动时执行一次
        .add_systems(Startup, (
            setup_camera,    // 设置3D相机和光照
            setup_ground,    // 创建地面平面
            setup_game_sounds, // 加载游戏音效资源
        ))
        
        // ===== 主菜单状态 =====
        .add_systems(
            // 进入主菜单状态时执行的一次性系统
            OnEnter(GameState::MainMenu),
            (
                setup_main_menu,     // 设置主菜单UI元素
                clear_player,        // 清除可能存在的玩家实体
                clear_platforms,     // 清除可能存在的平台实体
                despawn_scoreboard,  // 清除可能存在的计分板UI
            ),
        )
        .add_systems(
            // 主菜单状态下每帧更新的系统
            Update,
            (click_button,).run_if(in_state(GameState::MainMenu)), // 处理按钮点击事件
        )
        .add_systems(
            // 退出主菜单状态时执行的一次性系统
            OnExit(GameState::MainMenu),
            (despawn_screen::<OnMainMenuScreen>,), // 移除主菜单UI元素
        )
        
        // ===== 游戏进行状态 =====
        .add_systems(
            // 进入游戏进行状态时执行的一次性系统
            OnEnter(GameState::Playing),
            (
                clear_player,                   // 清除旧的玩家实体
                clear_platforms,                // 清除旧的平台实体
                despawn_scoreboard,             // 清除旧的计分板
                setup_first_platform.after(clear_platforms), // 设置第一个平台（注意依赖关系）
                setup_player.after(clear_player),           // 设置玩家（注意依赖关系）
                setup_scoreboard.after(despawn_scoreboard), // 设置计分板（注意依赖关系）
                reset_score,                    // 重置分数为0
                reset_prepare_jump_timer,       // 重置准备跳跃计时器
            ),
        )
        .add_systems(
            // 游戏进行状态下每帧更新的系统
            Update,
            (
                // 游戏核心逻辑系统，按特定顺序执行
                prepare_jump,                      // 更新准备跳跃计时器
                generate_next_platform,            // 生成下一个平台
                move_camera,                       // 相机跟随玩家移动
                player_jump,                       // 玩家跳跃核心逻辑
                update_scoreboard,                 // 更新分数显示
                animate_jump,                      // 执行跳跃动画
                animate_fall,                      // 执行摔落动画（如果需要）
                animate_player_accumulation,       // 玩家蓄力视觉效果
                animate_platform_accumulation.after(player_jump), // 平台蓄力效果（依赖跳跃逻辑）
                spawn_score_up_effect,             // 生成得分上升效果
                sync_score_up_effect,              // 同步得分效果位置到屏幕坐标
                shift_score_up_effect,             // 处理得分效果的上移动画
            )
                .run_if(in_state(GameState::Playing)), // 条件：仅在游戏进行状态执行
        )
        
        // ===== 游戏结束状态 =====
        .add_systems(
            // 进入游戏结束状态时执行的一次性系统
            OnEnter(GameState::GameOver), 
            (setup_game_over_menu,) // 设置游戏结束菜单UI
        )
        .add_systems(
            // 游戏结束状态下每帧更新的系统
            Update,
            (click_button,).run_if(in_state(GameState::GameOver)), // 处理按钮点击事件
        )
        .add_systems(
            // 退出游戏结束状态时执行的一次性系统
            OnExit(GameState::GameOver),
            (despawn_screen::<OnGameOverMenuScreen>,), // 移除游戏结束菜单UI
        );

    // 仅在非Web平台添加粒子效果动画系统
    // 为蓄力效果提供视觉反馈
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_systems(Update, animate_accumulation_particle_effect);
    }

    // 启动游戏主循环，开始运行所有注册的系统
    app.run();
}
