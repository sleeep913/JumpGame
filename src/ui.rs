use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::player::{JumpState, INITIAL_PLAYER_POS};

/// 游戏状态枚举，控制游戏流程的不同阶段
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum GameState {
    #[default]
    MainMenu,  // 主菜单界面
    Playing,   // 游戏进行中
    GameOver,  // 游戏结束界面
}

/// 游戏音效资源，管理所有游戏中的音频文件
#[derive(Debug, Resource)]
pub struct GameSounds {
    pub start: Handle<AudioSource>,       // 游戏开始音效
    pub accumulation: Handle<AudioSource>, // 蓄力音效
    pub fall: Handle<AudioSource>,         // 摔落音效
    pub success: Handle<AudioSource>,      // 成功跳跃音效
}

/// 菜单按钮功能组件，定义按钮的点击行为
#[derive(Component)]
pub enum MenuButtonAction {
    StartGame,       // 开始游戏
    RestartGame,     // 重新开始游戏
    BackToMainMenu,  // 返回主菜单
}

/// 标记主菜单界面元素的组件
#[derive(Component)]
pub struct OnMainMenuScreen;

/// 标记游戏结束菜单界面元素的组件
#[derive(Component)]
pub struct OnGameOverMenuScreen;

/// 游戏分数资源，跟踪当前游戏得分
#[derive(Debug, Resource)]
pub struct Score(pub u32);

/// 标记分数显示文本的组件
#[derive(Debug, Component)]
pub struct Scoreboard;

/// 飘分效果队列资源，存储待显示的飘分事件
#[derive(Debug, Resource)]
pub struct ScoreUpQueue(pub Vec<ScoreUpEvent>);

/// 飘分事件结构，包含飘分起始位置信息
#[derive(Debug)]
pub struct ScoreUpEvent {
    pub landing_pos: Vec3, // 着陆位置，用于显示飘分效果
}

/// 飘分效果组件，控制分数向上飘的动画效果
#[derive(Debug, Component)]
pub struct ScoreUpEffect(pub Vec3); // 当前飘分位置

/// 加载并设置游戏音效资源
/// 
/// 从资源目录加载所有必要的音效文件并注册为全局资源
pub fn setup_game_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameSounds {
        start: asset_server.load("sounds/start.mp3"),
        accumulation: asset_server.load("sounds/accumulation.mp3"),
        fall: asset_server.load("sounds/fall.mp3"),
        success: asset_server.load("sounds/success.mp3"),
    });
}

/// 设置主菜单界面
/// 
/// 创建主菜单布局，包含游戏标题和开始按钮
pub fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node { // 主容器节点，全屏覆盖
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnMainMenuScreen, // 标记为属于主菜单的元素
        ))
        .with_children(|parent| {
            parent
                .spawn((Node { // 垂直排列的内容容器
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|parent| {
                    // 标题
                    parent.spawn((ImageNode::new(
                        asset_server.load("image/title.png").into(),
                    ),));

                    // 开始按钮
                    parent.spawn((
                        Button,  // 按钮交互组件
                        Node { // 按钮样式
                            width: Val::Px(150.),
                            height: Val::Px(60.),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ImageNode::new(asset_server.load("image/btn_start.png").into()),
                        MenuButtonAction::StartGame, // 按钮功能标记
                    ));
                });
        });
}

/// 设置游戏结束菜单界面
/// 
/// 创建游戏结束布局，包含标题、返回按钮和重新开始按钮
pub fn setup_game_over_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node { // 主容器节点，全屏覆盖
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnGameOverMenuScreen, // 标记为属于游戏结束菜单的元素
        ))
        .with_children(|parent| {
            parent
                .spawn((Node { // 垂直排列的内容容器
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|parent| {
                    // 标题
                    parent.spawn((ImageNode::new(asset_server.load("image/title.png")),));

                    parent
                        .spawn((Node { // 水平排列的按钮容器
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|parent| {
                            // 返回按钮
                            parent.spawn((
                                Button, // 按钮交互组件
                                Node { // 按钮样式
                                    width: Val::Px(40.),
                                    height: Val::Px(40.),
                                    margin: UiRect::all(Val::Px(10.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ImageNode::new(asset_server.load("image/btn_home.png")),
                                MenuButtonAction::BackToMainMenu, // 按钮功能标记
                            ));

                            // 重新开始按钮
                            parent.spawn((
                                Button, // 按钮交互组件
                                Node { // 按钮样式
                                    width: Val::Px(150.),
                                    height: Val::Px(60.),
                                    margin: UiRect::all(Val::Px(10.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ImageNode::new(asset_server.load("image/btn_restart.png")),
                                MenuButtonAction::RestartGame, // 按钮功能标记
                            ));
                        });
                });
        });
}

/// 设置计分板界面
/// 
/// 在游戏界面左上角创建显示分数的文本元素
pub fn setup_scoreboard(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Text::new("Score: "), // 分数标签文本
            TextColor(Color::BLACK), // 文本颜色
            TextFont { // 文本字体设置
                font: asset_server.load("fonts/num.ttf"),
                font_size: 40.0,
                ..default()
            },
            Node { // 位置设置
                position_type: PositionType::Absolute, // 绝对定位
                top: Val::Px(30.0), // 距离顶部30像素
                left: Val::Px(30.0), // 距离左侧30像素
                ..default()
            },
        ))
        .with_child(( // 分数值文本子元素
            TextSpan::new("0"), // 初始分数
            TextColor(Color::BLACK), // 分数颜色
            TextFont { // 字体设置
                font: asset_server.load("fonts/num.ttf"),
                font_size: 40.0,
                ..default()
            },
            Scoreboard, // 标记为计分板元素
        ));
}

/// 更新计分板显示
/// 
/// 当分数资源发生变化时，更新UI中的分数显示
pub fn update_scoreboard(score: Res<Score>, mut span: Single<&mut TextSpan, With<Scoreboard>>) {
    if score.is_changed() { // 只有在分数变化时才更新
        span.0 = score.0.to_string(); // 将分数值转换为字符串更新显示
    }
}

/// 同步飘分效果与3D世界坐标
/// 
/// 将3D世界中的位置转换为屏幕坐标，更新飘分UI元素的位置
pub fn sync_score_up_effect(
    mut q_score_up_effect: Query<(&mut Node, &mut ScoreUpEffect)>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let (camera, camera_global_transform) = q_camera.single(); // 获取主摄像机
    let window = q_windows.single(); // 获取主窗口
    for (mut score_up_effect_style, score_up_effect) in &mut q_score_up_effect {
        // 将3D世界坐标转换为屏幕视口坐标
        let viewport_pos = camera
            .world_to_viewport(camera_global_transform, score_up_effect.0)
            .unwrap();
        // 更新UI元素位置，注意y轴需要翻转（屏幕坐标系与世界坐标系y轴方向相反）
        score_up_effect_style.top = Val::Px(window.resolution.height() - viewport_pos.y);
        score_up_effect_style.left = Val::Px(viewport_pos.x);
    }
}

/// 控制飘分效果向上移动
/// 
/// 实现飘分数字向上飘动并逐渐消失的动画效果
pub fn shift_score_up_effect(
    mut commands: Commands,
    mut q_score_up_effect: Query<(Entity, &mut TextColor, &mut ScoreUpEffect)>,
    time: Res<Time>,
) {
    for (entity, mut text_color, mut score_up_effect) in &mut q_score_up_effect {
        // 垂直方向向上移动
        score_up_effect.0.y += 1.0 * time.delta_secs();
        // 边移动边增加透明度，实现淡出效果
        let alpha = text_color.0.alpha();
        text_color.0.set_alpha(alpha * 0.97);
        
        // 当飘分到足够高度时，移除该元素
        if score_up_effect.0.y > INITIAL_PLAYER_POS.y + 1.2 {
            commands.entity(entity).despawn();
        }
    }
}

/// 创建飘分效果
/// 
/// 当跳跃完成时，从飘分队列中创建新的飘分UI元素
pub fn spawn_score_up_effect(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut score_up_queue: ResMut<ScoreUpQueue>,
    jump_state: Res<JumpState>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    // 只有当跳跃完成时才处理飘分效果
    if jump_state.completed {
        let window = q_windows.single();
        // 为队列中的每个飘分事件创建UI元素
        for score_up_event in score_up_queue.0.iter_mut() {
            let (camera, camera_global_transform) = q_camera.single();
            // 将3D世界坐标转换为屏幕坐标
            let viewport_pos = camera
                .world_to_viewport(camera_global_transform, score_up_event.landing_pos)
                .unwrap();
            
            // 创建飘分文本元素
            commands.spawn((
                Text::new("+1"), // 分数增量文本
                TextColor(Color::srgb(0.5, 0.5, 1.0)), // 文本颜色
                TextFont { // 字体设置
                    font: asset_server.load("fonts/num.ttf"),
                    font_size: 40.0,
                    ..default()
                },
                Node { // 位置设置
                    position_type: PositionType::Absolute,
                    top: Val::Px(window.resolution.height() - viewport_pos.y),
                    left: Val::Px(viewport_pos.x),
                    ..default()
                },
                ScoreUpEffect(score_up_event.landing_pos), // 飘分效果组件
            ));
        }
        // 清空队列，避免重复处理
        score_up_queue.0.clear();
    }
}

/// 处理按钮点击事件
/// 
/// 监听所有菜单按钮的点击事件，并根据按钮功能执行相应操作
pub fn click_button(
    mut interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &mut interaction_query {
        // 只有在按钮被按下时处理
        match *interaction {
            Interaction::Pressed => match menu_button_action {
                MenuButtonAction::StartGame => {
                    info!("StartGame button clicked");
                    next_game_state.set(GameState::Playing); // 切换到游戏进行状态
                }
                MenuButtonAction::RestartGame => {
                    info!("RestartGame button clicked");
                    next_game_state.set(GameState::Playing); // 切换到游戏进行状态
                }
                MenuButtonAction::BackToMainMenu => {
                    info!("BackToMainMenu button clicked");
                    next_game_state.set(GameState::MainMenu); // 切换到主菜单状态
                }
            },
            _ => {} // 忽略其他交互状态
        }
    }
}

/// 清理指定类型的UI界面元素
/// 
/// 通用函数，用于在状态切换时移除特定类型的UI元素
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive(); // 递归删除，确保子元素也被清理
    }
}

/// 清理计分板元素
/// 
/// 在游戏状态切换时移除计分板
pub fn despawn_scoreboard(mut commands: Commands, q_scoreboard: Query<Entity, With<Scoreboard>>) {
    for scoreboard in &q_scoreboard {
        commands.entity(scoreboard).despawn();
    }
}

/// 重置游戏分数
/// 
/// 在游戏重新开始时将分数重置为0
pub fn reset_score(mut score: ResMut<Score>) {
    score.0 = 0;
}
