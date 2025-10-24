// 导入玩家模块中的必要组件和常量
use crate::player::{FallState, JumpState, Player, INITIAL_PLAYER_POS};
// 导入后处理效果中的泛光效果
use bevy::core_pipeline::bloom::Bloom;
// 导入Bevy的主要组件
use bevy::prelude::*;

/// 相机初始位置常量
/// 设置为俯视角度：X=-5，Y=8，Z=5，提供良好的游戏视角
pub const INITIAL_CAMERA_POS: Vec3 = Vec3::new(-5.0, 8.0, 5.0);

/// 相机移动状态资源
/// 用于控制相机平滑跟随玩家的逻辑
#[derive(Debug, Resource)]
pub struct CameraMoveState {
    /// 相机每帧移动的步长向量
    step: Vec3,
    /// 记录玩家位置，用于检测移动
    player_pos: Vec3,
}

/// 为CameraMoveState实现默认初始化
impl Default for CameraMoveState {
    fn default() -> Self {
        Self {
            step: Vec3::ZERO,  // 初始步长为零向量
            player_pos: INITIAL_PLAYER_POS,  // 初始位置设为玩家初始位置
        }
    }
}

/// 设置游戏相机和光照
/// 
/// 此函数在游戏启动时执行，创建方向光和主相机
pub fn setup_camera(mut commands: Commands) {
    // 创建方向光（模拟太阳光）
    // TODO: 未来可以添加更复杂的阴影设置
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,  // 设置光照强度
            shadows_enabled: true,  // 启用阴影
            ..default()
        },
        // 设置光源位置和朝向
        Transform::from_xyz(2.0, 10.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 创建主相机
    commands.spawn((
        Camera3d::default(),  // 3D相机组件
        // 设置相机初始位置和朝向（俯视视角）
        Transform::from_translation(INITIAL_CAMERA_POS).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            hdr: true,  // 启用HDR渲染，获得更好的光照效果
            ..default()
        },
        Bloom::default(),  // 添加泛光效果，增强视觉体验
    ));
}

/// 设置游戏地面
/// 
/// 创建一个巨大的平面作为游戏的地面
pub fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 创建地面平面
    commands.spawn((
        Mesh3d(
            meshes.add(
                // 创建一个非常大的平面（1000000x1000000）以确保玩家不会走到边界
                Plane3d::new(Vec3::Y, Vec2::new(1000000.0, 1000000.0))
                    .mesh()
                    .size(1000000.0, 1000000.0),
            ),
        ),
        // 设置地面材质为浅粉色
        MeshMaterial3d(materials.add(Color::srgb(0.95, 0.87, 0.88))),
    ));
}

/// 相机跟随玩家移动的系统
/// 
/// 实现相机平滑跟随玩家的功能，只在玩家不跳跃或不摔落时移动
pub fn move_camera(
    q_player: Query<&Transform, With<Player>>,  // 查询玩家变换组件
    mut q_camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,  // 查询相机变换组件
    mut camera_move_state: ResMut<CameraMoveState>,  // 相机移动状态资源
    jump_state: Res<JumpState>,  // 跳跃状态资源
    fall_state: Res<FallState>,  // 摔落状态资源
) {
    // 只有当跳跃和摔落动画都完成时，才移动相机
    // 这样可以避免在跳跃过程中相机跟随，影响玩家体验
    if jump_state.completed && fall_state.completed {
        let player = q_player.single();
        let mut camera = q_camera.single_mut();
        
        // 计算相机应该到达的目标位置
        // 保持与玩家的相对位置不变
        let camera_destination = INITIAL_CAMERA_POS + player.translation;

        // 检测玩家是否移动了足够的距离（大于0.1单位）
        // 如果移动了，则重新计算相机移动步长
        if camera_move_state.player_pos.distance(player.translation) > 0.1 {
            let delta = camera_destination - camera.translation;
            // 步长设置为总距离的5%，实现平滑过渡效果
            camera_move_state.step = 0.05 * delta;
            // 更新记录的玩家位置
            camera_move_state.player_pos = player.translation;
        }

        // 如果相机还没到达目标位置，则继续移动
        // 使用步长向量的长度作为阈值，避免无限接近但永远无法到达的情况
        if camera.translation.distance(camera_destination) > Vec3::ZERO.distance(camera_move_state.step) {
            camera.translation = camera.translation + camera_move_state.step;
        }
    }
}
