use crate::player::{FallState, JumpState, Player, INITIAL_PLAYER_POS};
use bevy::core_pipeline::bloom::Bloom;
use bevy::prelude::*;

// 定义相机的初始位置
pub const INITIAL_CAMERA_POS: Vec3 = Vec3::new(-5.0, 8.0, 5.0);

// 相机移动状态资源，用于记录相机的移动步长和玩家位置
#[derive(Debug, Resource)]
pub struct CameraMoveState {
    step: Vec3,       // 相机的移动步长
    player_pos: Vec3, // 玩家的当前位置
}

// 为 CameraMoveState 实现默认值
impl Default for CameraMoveState {
    fn default() -> Self {
        Self {
            step: Vec3::ZERO,          // 初始步长为零
            player_pos: INITIAL_PLAYER_POS, // 初始玩家位置为默认值
        }
    }
}

// 设置相机和光源
pub fn setup_camera(mut commands: Commands) {
    // 创建方向光（模拟太阳光）
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0, // 光照强度
            shadows_enabled: true, // 启用阴影
            ..default()
        },
        Transform::from_xyz(2.0, 10.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y), // 光源的位置和朝向
    ));

    // 创建相机
    commands.spawn((
        Camera3d::default(), // 默认的 3D 相机
        Transform::from_translation(INITIAL_CAMERA_POS).looking_at(Vec3::ZERO, Vec3::Y), // 相机的位置和朝向
        Camera {
            hdr: true, // 启用高动态范围渲染
            ..default()
        },
        Bloom::default(), // 启用泛光效果
    ));
}

// 设置地面
pub fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 创建地面
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::new(Vec3::Y, Vec2::new(1000000.0, 1000000.0)) // 创建一个平面
                    .mesh()
                    .size(1000000.0, 1000000.0), // 设置平面的大小
            ),
        ),
        MeshMaterial3d(materials.add(Color::srgb(0.95, 0.87, 0.88))), // 设置地面的材质颜色
    ));
}

// 相机跟随玩家移动的逻辑
pub fn move_camera(
    q_player: Query<&Transform, With<Player>>, // 查询玩家的位置
    mut q_camera: Query<&mut Transform, (With<Camera>, Without<Player>)>, // 查询相机的位置
    mut camera_move_state: ResMut<CameraMoveState>, // 相机移动状态资源
    jump_state: Res<JumpState>, // 玩家的跳跃状态
    fall_state: Res<FallState>, // 玩家的摔落状态
) {
    // 如果玩家没有在跳跃或摔落，则移动相机
    if jump_state.completed && fall_state.completed {
        let player = q_player.single(); // 获取玩家的位置
        let mut camera = q_camera.single_mut(); // 获取相机的位置
        let camera_destination = INITIAL_CAMERA_POS + player.translation; // 计算相机的目标位置

        // 如果玩家移动了，重新计算相机的移动步长
        if camera_move_state.player_pos.distance(player.translation) > 0.1 {
            let delta = camera_destination - camera.translation; // 计算相机与目标位置的差值
            camera_move_state.step = 0.05 * delta; // 设置移动步长为差值的 5%
            camera_move_state.player_pos = player.translation; // 更新玩家的位置
        }

        // 如果相机与目标位置的距离大于步长，则移动相机
        if camera.translation.distance(camera_destination)
            > Vec3::ZERO.distance(camera_move_state.step)
        {
            camera.translation = camera.translation + camera_move_state.step; // 移动相机
        }
    }
}
