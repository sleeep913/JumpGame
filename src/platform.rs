// 导入Bevy游戏引擎的主要组件
use bevy::prelude::*;
// 导入随机数生成库，用于随机生成平台属性
use rand::Rng;


/// 标记组件：表示当前玩家站立的平台
#[derive(Debug, Component)]
pub struct CurrentPlatform;

/// 标记组件：表示下一个需要跳跃的目标平台
#[derive(Debug, Component)]
pub struct NextPlatform;

/// 平台形状枚举，表示不同类型的平台
#[derive(Debug, Component)]
pub enum PlatformShape {
    /// 方形平台
    Box,
    /// 圆柱形平台
    Cylinder,
}

impl PlatformShape {
    /// 根据平台形状生成对应的网格模型
    pub fn mesh(&self) -> Mesh {
        match self {
            // 生成一个1.5x1.0x1.5大小的立方体
            Self::Box => Mesh::from(Cuboid::new(1.5, 1.0, 1.5)),
            // 生成一个半径0.75，高度1.0的圆柱体
            Self::Cylinder => Mesh::from(Cylinder::new(0.75, 1.0)),
        }
    }
    
    /// 判断玩家是否成功落到平台上
    /// 
    /// # 参数
    /// - `platform_pos`: 平台的位置坐标
    /// - `landing_pos`: 玩家的落地点坐标
    /// 
    /// # 返回值
    /// 如果落地点在平台范围内返回true，否则返回false
    pub fn is_landed_on_platform(&self, platform_pos: Vec3, landing_pos: Vec3) -> bool {
        // 调试输出，实际游戏中可以移除
        dbg!(platform_pos);
        dbg!(landing_pos);
        
        match self {
            // 对于方形平台，判断落地点是否在平台的X和Z轴范围内
            Self::Box => {
                (landing_pos.x - platform_pos.x).abs() < 1.5 / 2.0
                    && (landing_pos.z - platform_pos.z).abs() < 1.5 / 2.0
            }
            // 对于圆柱形平台，使用简化的碰撞检测（方形边界）
            Self::Cylinder => {
                (landing_pos.x - platform_pos.x).abs() < 0.75
                    && (landing_pos.z - platform_pos.z).abs() < 0.75
            }
        }
    }
    
    /// 判断玩家是否接触到平台（用于检测边缘碰撞）
    /// 
    /// # 参数
    /// - `platform_pos`: 平台的位置坐标
    /// - `landing_pos`: 玩家的位置坐标
    /// - `player_radius`: 玩家的半径（用于碰撞检测）
    /// 
    /// # 返回值
    /// 如果玩家接触到平台返回true，否则返回false
    pub fn is_touched_player(
        &self,
        platform_pos: Vec3,
        landing_pos: Vec3,
        player_radius: f32,
    ) -> bool {
        match self {
            // 方形平台的接触检测，包含玩家半径
            Self::Box => {
                (landing_pos.x - platform_pos.x).abs() < (1.5 / 2.0 + player_radius)
                    && (landing_pos.z - platform_pos.z).abs() < (1.5 / 2.0 + player_radius)
            }
            // 圆柱形平台的接触检测，包含玩家半径
            Self::Cylinder => {
                (landing_pos.x - platform_pos.x).abs() < (0.75 + player_radius)
                    && (landing_pos.z - platform_pos.z).abs() < (0.75 + player_radius)
            }
        }
    }
}

/// 生成一个随机属性的平台
/// 
/// # 参数
/// - `commands`: 命令实体，用于生成平台实体
/// - `meshes`: 网格资源，用于创建平台模型
/// - `materials`: 材质资源，用于创建平台材质
/// - `pos`: 平台的位置坐标
/// - `component`: 平台需要添加的组件（CurrentPlatform或NextPlatform）
fn spawn_rand_platform<T: Component>(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    component: T,
) {
    // 随机生成平台形状
    let platform_shape = rand_platform_shape();
    
    // 创建平台实体
    commands.spawn((
        Mesh3d(meshes.add(platform_shape.mesh())),  // 添加网格组件
        MeshMaterial3d(materials.add(rand_platform_color())),  // 添加材质组件
        Transform::from_translation(pos),  // 设置位置
        platform_shape,  // 添加形状组件
        component,  // 添加平台类型组件
    ));
}

/// 设置游戏开始时的第一个平台
/// 
/// 在原点位置生成一个作为当前平台的实体
pub fn setup_first_platform(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_rand_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 0.5, 0.0),  // 在(0, 0.5, 0)位置生成（Y=0.5使平台顶面在Y=1.0）
        CurrentPlatform,
    );
}

/// 生成下一个目标平台
/// 
/// 当没有下一个平台时，在当前平台的X或Z方向随机生成一个新平台
pub fn generate_next_platform(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_current_platform: Query<&Transform, With<CurrentPlatform>>,
    q_next_platform: Query<Entity, With<NextPlatform>>,
) {
    // 只有当没有下一个平台时才生成新的
    if q_next_platform.is_empty() {
        let current_platform = &q_current_platform.single();
        let mut rng = rand::thread_rng();
        
        // 随机生成平台间的距离（2.5到4.0之间）
        let rand_distance = rng.gen_range(2.5..4.0);
        
        // 50%概率在X轴方向，50%概率在Z轴方向生成新平台
        let next_pos = if rng.gen_bool(0.5) {
            Vec3::new(
                current_platform.translation.x + rand_distance,  // X轴正方向
                0.5,  // 保持相同高度
                current_platform.translation.z,
            )
        } else {
            Vec3::new(
                current_platform.translation.x,
                0.5,
                current_platform.translation.z - rand_distance,  // Z轴负方向
            )
        };

        // 生成新平台并标记为NextPlatform
        spawn_rand_platform(
            &mut commands,
            &mut meshes,
            &mut materials,
            next_pos,
            NextPlatform,
        );
    }
}

/// 平台蓄力动画效果
/// 
/// 当玩家蓄力时，当前平台会被压缩，模拟蓄力效果
pub fn animate_platform_accumulation(
    accumulator: Res<Accumulator>,  // 蓄力状态资源
    mut q_current_platform: Query<&mut Transform, With<CurrentPlatform>>,  // 当前平台查询
    time: Res<Time>,  // 时间资源，用于帧间平滑过渡
) {
    let mut current_platform = q_current_platform.single_mut();
    
    match accumulator.0 {
        // 正在蓄力时，平台Y轴缩放逐渐减小（压缩效果）
        Some(_) => {
            current_platform.scale.y = 
                (current_platform.scale.y - 0.15 * time.delta_secs()).max(0.6);  // 最小缩放到0.6
        }
        // 蓄力结束时，平台恢复原状
        None => {
            // TODO: 后续可以添加回弹效果，使平台恢复时更有弹性
            current_platform.scale = Vec3::ONE;
        }
    }
}

/// 清除所有平台实体
/// 
/// 用于状态切换时清理场景
pub fn clear_platforms(mut commands: Commands, q_platforms: Query<Entity, With<PlatformShape>>) {
    for platform in &q_platforms {
        commands.entity(platform).despawn();
    }
}

/// 随机生成平台颜色
/// 
/// 使用RGB随机值生成平台颜色
fn rand_platform_color() -> Color {
    let mut rng = rand::thread_rng();
    Color::srgb(rng.gen(), rng.gen(), rng.gen())  // 随机生成RGB值
}

/// 随机生成平台形状
/// 
/// 50%概率生成方形平台，50%概率生成圆柱形平台
fn rand_platform_shape() -> PlatformShape {
    let mut rng = rand::thread_rng();
    let selection = rng.gen_range(0..2);
    match selection {
        0 => PlatformShape::Box,
        1 => PlatformShape::Cylinder,
        _ => PlatformShape::Box,  // 默认情况，避免模式匹配不完整的警告
    }
}
