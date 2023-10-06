use std::fmt::{Display, Formatter, write};
use bytemuck_derive::{AnyBitPattern, Pod, Zeroable};
use crate::r#struct::offsets::Offsets;
use serde_big_array::BigArray;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde::de::{Unexpected, Visitor};
use serde_with::serde_as;
use crate::constants;


struct Context {
    name: String,
    offsets: Offsets,
    state: GameState,
    client: u32,
    engine: u32,
}

enum Module {
    Client(u32),
    Engine(u32),
}


impl Context {
    pub fn get_address(self, module: Module, offset: u32) -> u32 {
        let module = match module {
            Module::Client(addr) => addr,
            Module::Engine(addr) => addr,
        };

        module + offset
    }
}


pub struct PlayerEntityNon {
    base_address: u32,
    offsets: Offsets,
}

// impl PlayerEntityNon {
// pub fn get
// }

// player entity is 4092 (0x0FFC) bytes long
// #[derive(Deserialize, Debug)]
// pub struct PlayerEntitySmol {
//     pad_base: [u8; 244],
//     // first 244 bytes of nothing (0x00 - 0xF0)
//     #[serde(deserialize_with = "deserialize_team")]
//     team: Team,
//     // 0xF4 1 - Spec, 2 - T, 3 - CT
//     #[serde(deserialize_with = "deserialize_team")]
//     team2: Team,
//     // 0xF8 1 - Spec, 2 - T, 3 - CT
//     pad_00fc: [u8; 4],
//     // next 4 bytes of nothing (0xFC)
//     health: u32,
//     // 0x100
//     #[serde(deserialize_with = "deserialize_flags")]
//     flags: MovementFlags,
//     pad_0105: [u8; 51],
//     // next 51 bytes of nothing (0x105 - 0x134)
//     feet_origin: Vector, // 0x138
// }


pub struct PlayerEntity {
    pub health: u16,
    team: Team,
    flags: u32,
    feet_origin: Vector3,
    head_origin: Vector3,
    movement_state: MovementState,
    velocity_vector: Vector3,
    dormant: bool,
    immune: bool,
    visible: bool,
    xhair_id: Option<u32>,
}

impl PlayerEntity {
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn is_enemy(&self, other: &PlayerEntity) -> bool {
        self.team != other.team
    }

    pub fn is_immune(&self) -> bool {
        self.immune
    }

    pub fn is_moving(&self) -> bool {
        self.velocity_vector.x != 0.0 || self.velocity_vector.y != 0.0 || self.velocity_vector.z != 0.0
    }

    pub fn on_ground(&self) -> bool {
        self.flags & MovementBitwiseOps::OnGround as u32 != 0
    }

    pub fn is_aiming_at_valid_enemy(&self, entities: &[PlayerEntity]) -> bool {
        let Some(xhair_id) = self.xhair_id else {
            return false
        };


        let range = 1..constants::MAX_PLAYERS;
        if ! range.contains(&xhair_id) || xhair_id == 0 {
            println!("Invalid xhair id: {xhair_id}", xhair_id = xhair_id);
            return false
        }

        let player_index = (xhair_id - 1) as usize;
        let player_being_aimed_at = &entities[player_index];


        if player_being_aimed_at.is_dormant() {
            println!("Player being aimed at is dormant");
            return false
        }

        if player_being_aimed_at.is_immune() {
            println!("Player being aimed at is immune");
            return false
        }

        println!("IMmune: {immune}", immune = player_being_aimed_at.is_immune());

        true


    }



    pub fn is_dormant(&self) -> bool {
        self.dormant
    }

    pub fn from_raw(raw: &PlayerEntityRaw, xhair_id: Option<u32>) -> Self {
        let head_pos = raw.m_vecOrigin.add(&raw.m_vecViewOffset);

        PlayerEntity {
            team: match raw.team {
                1 => Team::Spec,
                2 => Team::T,
                3 => Team::CT,
                _ => Team::Spec
            },
            movement_state: match raw.move_state {
                0 => MovementState::None,
                1 => MovementState::Isometric,
                2 => MovementState::Walk,
                3 => MovementState::Step,
                4 => MovementState::Fly,
                5 => MovementState::FlyGravity,
                6 => MovementState::VPhysics,
                7 => MovementState::Push,
                8 => MovementState::NoClip,
                9 => MovementState::Ladder,
                10 => MovementState::Observer,
                _ => MovementState::Custom
            },
            health: raw.m_iHealth as u16,
            flags: raw.m_fFlags as u32,
            head_origin: head_pos,
            feet_origin: raw.m_vecOrigin,
            velocity_vector: raw.m_vecVelocity,
            dormant: raw.dormant == 1,
            visible: raw.m_bSpottedByMask == 1,
            immune: raw.m_bGunGameImmunity == 1,
            xhair_id,
        }
    }

    pub fn find_closet_to_given_head(&self, other: &PlayerEntity) -> f32 {
        let head_pos = other.head_origin;
        let feet_pos = self.feet_origin;
        let head_pos = Vector3 {
            x: head_pos.x,
            y: head_pos.y,
            z: head_pos.z,
        };
        let feet_pos = Vector3 {
            x: feet_pos.x,
            y: feet_pos.y,
            z: feet_pos.z,
        };
        let distance = head_pos.sub(&feet_pos);
        let distance = distance.mul(&distance);
        let distance = distance.x + distance.y + distance.z;
        distance.sqrt()
    }

    pub fn from_raw_vec(raw: &[u8], xhair_id: Option<u32>) -> Self {
        let raw = bytemuck::from_bytes::<PlayerEntityRaw>(raw);
        PlayerEntity::from_raw(raw, xhair_id)
    }
}

impl Display for PlayerEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "bit flags = {:#08b} team: {team}, health: {health}, velocity = {velocity}, feet_pos = {feet_pos}, dormant: {dormant}, head_origin {head_origin} visible: {visible}, move_state:  {move_state}, xhair_id: {:#?} ",
               self.flags,
               self.xhair_id,
               team = self.team,
               health = self.health,
               feet_pos = self.feet_origin,
               dormant = self.is_dormant(),
               visible = self.visible,
               velocity = self.velocity_vector,
               head_origin = self.head_origin,
               move_state = self.movement_state,
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone, AnyBitPattern)]
pub struct PlayerEntityRaw {
    _0x70: [u8; 32],
    m_bFreezePeriod: u8,
    _0x74: [u8; 79],
    m_clrRender: u32,
    m_bIsQueuedMatchmaking: u8,
    _0x_f4: [u8; 7],
    m_bIsValveDS: u8,
    _0x100: [u8; 112],
    dormant: u8,
    _0x1000: [u8; 6],
    team: u8,
    _0x104: [u8; 8],
    pub m_iHealth: i32,
    m_fFlags: i32,
    m_vecViewOffset: Vector3,
    m_vecVelocity: Vector3,
    _0x14C: [u8; 24],
    m_vecOrigin: Vector3,
    _0x258: [u8; 8],
    m_hOwnerEntity: i32,
    _0x25C: [u8; 264],
    m_nModelIndex: u32,
    move_state: u32,
    _0x268: [u8; 2],
    m_lifeState: u8,
    _0x320: [u8; 8],
    m_flSimulationTime: f32,
    _0x444: [u8; 180],
    m_Collision: u32,
    _0x474: [u8; 288],
    m_rgflCoordinateFrame: u32,
    _0x93D: [u8; 44],
    m_CollisionGroup: u32,
    _0x980: [u8; 1221],
    m_bSpotted: u8,
    _0x9A5: [u8; 66],
    m_bSpottedByMask: u8,
    _0x9D8: [u8; 36],
    m_bBombPlanted: u8,
    _0x9D9: [u8; 50],
    m_bUseCustomAutoExposureMin: u8,
    m_bUseCustomAutoExposureMax: u8,
    m_bUseCustomBloomScale: u8,
    _0x9E0: [u8; 1],
    m_flCustomAutoExposureMin: f32,
    m_flCustomAutoExposureMax: f32,
    m_flCustomBloomScale: f32,
    _0x1328: [u8; 792],
    m_SurvivalRules: u32,
    _0x1A84: [u8; 1572],
    m_SurvivalGameRuleDecisionTypes: u32,
    _0x1B88: [u8; 1880],
    m_iCompetitiveRanking: i32,
    _0x268C: [u8; 256],
    m_iCompetitiveWins: i32,
    _0x2690: [u8; 2816],
    m_nForceBone: u32,
    m_iMostRecentModelBoneCounter: i32,
    _0x2928: [u8; 20],
    m_dwBoneMatrix: u32,
    _0x2990: [u8; 636],
    m_flLastBoneSetupTime: f32,
    _0x2994: [u8; 100],
    m_bBombTicking: u8,
    _0x29A0: [u8; 3],
    m_nBombSite: u32,
    _0x29A4: [u8; 8],
    m_flC4Blow: f32,
    m_flTimerLength: f32,
    _0x29BC: [u8; 16],
    m_flDefuseLength: f32,
    m_flDefuseCountDown: f32,
    m_bBombDefused: u8,
    _0x29D0: [u8; 3],
    m_hBombDefuser: i32,
    _0x29DC: [u8; 8],
    m_nViewModelIndex: u32,
    _0x2D80: [u8; 8],
    m_hOwner: i32,
    _0x2E08: [u8; 928],
    m_flNextAttack: f32,
    _0x2F08: [u8; 132],
    m_hMyWeapons: i32,
    _0x2FBA: [u8; 252],
    m_hActiveWeapon: i32,
    _0x2FBC: [u8; 174],
    m_iItemDefinitionIndex: u8,
    _0x2FCC: [u8; 1],
    m_iEntityQuality: i32,
    _0x2FD0: [u8; 12],
    m_Local: u32,
    m_iItemIDHigh: u8,
    _0x3030: [u8; 7],
    m_iAccountID: i32,
    _0x303C: [u8; 84],
    m_viewPunchAngle: u32,
    _0x3048: [u8; 8],
    m_aimPunchAngle: Vector3,
    m_aimPunchAngleVel: i32,
    m_szCustomName: u32,
    _0x31D4: [u8; 384],
    m_OriginalOwnerXuidLow: u32,
    m_OriginalOwnerXuidHigh: u32,
    m_nFallbackPaintKit: u32,
    m_nFallbackSeed: u32,
    m_flFallbackWear: f32,
    m_nFallbackStatTrak: u32,
    m_thirdPersonViewAngles: u32,
    _0x31F8: [u8; 8],
    m_iFOV: i32,
    m_iFOVStart: i32,
    _0x3268: [u8; 76],
    m_flNextPrimaryAttack: f32,
    _0x3274: [u8; 28],
    m_iState: i32,
    _0x32B5: [u8; 8],
    m_iClip1: i32,
    _0x3308: [u8; 61],
    m_bInReload: u8,
    _0x333C: [u8; 82],
    m_hViewModel: i32,
    _0x3340: [u8; 48],
    m_iDefaultFOV: i32,
    m_fAccuracyPenalty: f32,
    _0x339C: [u8; 68],
    m_iObserverMode: i32,
    _0x3400: [u8; 16],
    m_hObserverTarget: i32,
    _0x3440: [u8; 96],
    m_bStartedArming: u8,
    _0x35C4: [u8; 63],
    m_nTickBase: u32,
    _0x9974: [u8; 384],
    m_szLastPlaceName: u32,
    _0x997C: [u8; 25516],
    m_bIsScoped: u8,
    _0x9990: [u8; 7],
    m_bIsDefusing: u8,
    _0x9ADC: [u8; 19],
    pub m_bGunGameImmunity: u8,
    _0x103E0: [u8; 331],
    m_flLowerBodyYawTarget: f32,
    _0x1046C: [u8; 26880],
    pub m_iShotsFired: i32,
    _0x10470: [u8; 136],
    m_flFlashMaxAlpha: f32,
    m_flFlashDuration: f32,
    _0x117C0: [u8; 20],
    m_iGlowIndex: i32,
    _0x117CC: [u8; 4916],
    m_bHasHelmet: u8,
    _0x117D0: [u8; 11],
    pub m_ArmorValue: u32,
    m_angEyeAnglesX: u32,
    m_angEyeAnglesY: u32,
    _0x11838: [u8; 4],
    pub m_bHasDefuser: u8,
}


impl Display for PlayerEntityRaw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "team: {}, health: {}, feet_pos = {} movement_flags = {}, defuser = {}, shots_fired = {}, gun_game_immunity = {}, dormant = {}",
               self.team,
               self.m_iHealth,
               self.m_vecOrigin,
               self.move_state,
               self.m_bHasDefuser,
               self.m_iShotsFired,
               self.m_bGunGameImmunity,
               self.dormant
        )
    }
}

#[derive(Copy, Clone, AnyBitPattern)]
struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vector3 {
    pub fn add(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn sub(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn mul(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}


#[derive(Deserialize, Debug)]
struct Vector2 {
    x: f32,
    y: f32,
}

impl Display for Vector2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{x}, {y}]", x = self.x, y = self.y)
    }
}

impl Display for Vector3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{x}, {y}, {z}]", x = self.x, y = self.y, z = self.z)
    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Team {
    Spec,
    T,
    CT,
}

impl Display for MovementFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MovementFlags::Jump => write!(f, "Jump"),
            MovementFlags::OnGround => write!(f, "Still"),
            MovementFlags::JumpCrouch => write!(f, "Jump Crouch"),
            MovementFlags::Crouching => write!(f, "Crouching"),
            MovementFlags::Unknown => write!(f, "Unknown"),
            MovementFlags::Ladder => write!(f, "Ladder"),
        }
    }
}


enum MovementBitwiseOps {
    OnGround = 1 << 0,
    Ducking = 1 << 1,
    AnimDucking = 1 << 2,
    WaterJump = 1 << 3,
    OnTrain = 1 << 4,
    InRain = 1 << 5,
    Frozen = 1 << 6,
    AtControls = 1 << 7,
    Client = 1 << 8,
    FakeClient = 1 << 9,
    InWater = 1 << 10,
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Team::Spec => write!(f, "Spec"),
            Team::T => write!(f, "T"),
            Team::CT => write!(f, "CT"),
        }
    }
}

#[derive(Debug)]
pub enum MovementFlags {
    Jump,
    OnGround,
    Crouching,
    JumpCrouch,
    Ladder,
    Unknown,
}

#[derive(Deserialize, Debug)]
enum MovementState {
    None,
    Isometric,
    Walk,
    Step,
    Fly,
    FlyGravity,
    VPhysics,
    Push,
    NoClip,
    Ladder,
    Observer,
    Custom,
}

impl Display for MovementState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MovementState::None => write!(f, "None"),
            MovementState::Isometric => write!(f, "Isometric"),
            MovementState::Walk => write!(f, "Walk"),
            MovementState::Step => write!(f, "Step"),
            MovementState::Fly => write!(f, "Fly"),
            MovementState::FlyGravity => write!(f, "FlyGravity"),
            MovementState::VPhysics => write!(f, "VPhysics"),
            MovementState::Push => write!(f, "Push"),
            MovementState::NoClip => write!(f, "No Clip"),
            MovementState::Ladder => write!(f, "Ladder"),
            MovementState::Observer => write!(f, "Observer"),
            MovementState::Custom => write!(f, "Custom"),
        }
    }
}


#[derive(PartialEq)]
pub enum GameState {
    None = 0,
    Challenge = 1,
    Connected = 2,
    New = 3,
    PreSpawn = 4,
    Spawn = 5,
    FullConnected = 6,
    ChangeLevel = 7,
}

impl GameState {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => GameState::None,
            1 => GameState::Challenge,
            2 => GameState::Connected,
            3 => GameState::New,
            4 => GameState::PreSpawn,
            5 => GameState::Spawn,
            6 => GameState::FullConnected,
            7 => GameState::ChangeLevel,
            _ => panic!("Invalid game state")
        }
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameState::None => write!(f, "None"),
            GameState::Challenge => write!(f, "Challenge"),
            GameState::Connected => write!(f, "Connected"),
            GameState::New => write!(f, "New"),
            GameState::PreSpawn => write!(f, "Pre Spawn"),
            GameState::Spawn => write!(f, "Spawn"),
            GameState::FullConnected => write!(f, "Full Connected"),
            GameState::ChangeLevel => write!(f, "Change Level"),
        }
    }
}