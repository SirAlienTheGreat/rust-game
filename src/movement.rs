pub(crate) mod movement {
    const PLAYER_SPEED:f32 = 2.5;
    const DEADZONE:f32 = 0.10;

    use bevy::input::mouse::MouseMotion;
    use bevy::{prelude::*};
    use bevy_rapier3d::prelude::*;

    use crate::setup_world;

    pub(crate) fn controls (
            time: Res<Time>,
            mut query: Query<(&mut setup_world::setup_objects::ControllingButWithInfo, &mut Velocity, Entity), With<setup_world::setup_objects::Controlling>>,
            keyboard_input: Res<Input<KeyCode>>,
            mut motion_evr: EventReader<MouseMotion>,
            axes: Res<Axis<GamepadAxis>>,
            buttons: Res<Input<GamepadButton>>,
            my_gamepad: Option<Res<MyGamepad>>,
            mut collision_events: EventReader<CollisionEvent>,
            rapier_context: Res<RapierContext>,
            ){

        for  (mut player_info, mut velocity, entity) in &mut query{

            // slowing down
            velocity.linvel.x = velocity.linvel.x/2.0;
            velocity.linvel.z = velocity.linvel.z/2.0;

            let mut input_theta:f32 = 0.0;
            let mut speed = 1.0;
            let mut moving = false;

            let mut dashing = false;


            if collision_events.len() >= 1 {
                player_info.has_hit_object = true;
            }
            for i in collision_events.iter() {
                println!("Collision event: {:?}",i);
                match i {
                    CollisionEvent::Started(_, _, _) => {
                        player_info.objects_hit+=1;
                    },
                    CollisionEvent::Stopped(_,_, _) => {
                        player_info.objects_hit-=1;
                    },
                }
            }


            if rapier_context.contacts_with(entity).count() >= 1 {
                player_info.has_contacts = true;
                let mut colliders = vec![];
                for i in rapier_context.contacts_with(entity) {
                    colliders.push(i.collider2())
                }
            } else {
                player_info.has_contacts = false;
            }

            // Controller controls
            let gamepad = if let Some(ref gp) = my_gamepad {
                // a gamepad is connected, we have the id
                Some(gp.0)
            } else {
                None
            };
            match gamepad {
                Some(gamepad) => {
                    // If a gamepad exists, this code is run.
                
                    let axis_lx = GamepadAxis {
                        gamepad, axis_type: GamepadAxisType::LeftStickX
                    };
                    let axis_ly = GamepadAxis {
                        gamepad, axis_type: GamepadAxisType::LeftStickY
                    };

                
                    //Reading Left stick
                    let x = axes.get(axis_lx).unwrap_or(0.0_f32);
                    let y = axes.get(axis_ly).unwrap_or(0.0_f32);

                    input_theta = 2.0*((y/(x + (x*x + y*y).sqrt())).atan());

                    if y == 0.0 && x < 0.0 {
                        input_theta = std::f32::consts::PI;
                    }

                    if !input_theta.is_nan(){
                        moving = true;
                        speed = (x*x + y*y).sqrt();
                        if speed > 1.0 {
                            speed = 1.0;
                        }
                    } else {
                        input_theta = 0.0;
                    }

                    if speed <= DEADZONE {
                        moving = false;
                    }

                    // camera
                    //horizontal
                    let axis_rx = GamepadAxis {
                        gamepad, axis_type: GamepadAxisType::RightStickX
                    };
                    let c_x = axes.get(axis_rx).unwrap_or(0.0_f32);

                    player_info.theta -= 2.0 * c_x * time.delta_seconds();

                    //vertical
                    let axis_ry = GamepadAxis {
                        gamepad, axis_type: GamepadAxisType::RightStickY
                    };
                    let c_y = axes.get(axis_ry).unwrap_or(0.0_f32);

                    if c_y> 0.0 && player_info.v_theta < 0.45*std::f32::consts::PI {
                        player_info.v_theta +=  2.0 * c_y * time.delta_seconds();
                    }
                    if c_y < 0.0 && player_info.v_theta > 0.0 {
                        player_info.v_theta +=  2.0 * c_y * time.delta_seconds();
                    }
                
                    // Jumping
                    let jump_button = GamepadButton {
                        gamepad, button_type: GamepadButtonType::South
                    };
        
                    if buttons.pressed(jump_button) && player_info.objects_hit >=1{
                        velocity.linvel.y = 10.0;
                        player_info.has_hit_object = false;
                    }

                    // Fast fall
                    let fast_fall_button = GamepadButton {
                        gamepad, button_type: GamepadButtonType::RightTrigger2
                    };

                    if buttons.pressed(fast_fall_button)&& player_info.objects_hit <=1{
                        velocity.linvel.y = -30.0;
                    }

                    // Dash
                    let dash_button = GamepadButton {
                        gamepad, button_type: GamepadButtonType::West
                    };

                    dashing = buttons.just_pressed(dash_button)
                
                }
                None => {},
            }

            // Keyboard controls

            if keyboard_input.pressed(KeyCode::A){
                input_theta = input_theta + std::f32::consts::PI;
                moving = true;
            }

            if keyboard_input.pressed(KeyCode::D){
                input_theta = input_theta;
                moving = true;
            }

            if keyboard_input.pressed(KeyCode::W){
                input_theta = input_theta + 0.5*std::f32::consts::PI;
                moving = true;
            }

            if keyboard_input.pressed(KeyCode::S){
                input_theta = input_theta + 1.5*std::f32::consts::PI;
                moving = true;
            }

            if keyboard_input.pressed(KeyCode::A) && keyboard_input.pressed(KeyCode::W) {
                input_theta = 0.75*std::f32::consts::PI;
                moving = true;
            }
        
            if keyboard_input.pressed(KeyCode::D) && keyboard_input.pressed(KeyCode::W) {
                input_theta = 0.25*std::f32::consts::PI;
            
                moving = true;
            }

            if keyboard_input.pressed(KeyCode::A) && keyboard_input.pressed(KeyCode::S) {
                input_theta = 1.25*std::f32::consts::PI;
                moving = true;
            }
        
            if keyboard_input.pressed(KeyCode::D) && keyboard_input.pressed(KeyCode::S) {
                input_theta = 1.75*std::f32::consts::PI;
                moving = true;
            }

            input_theta += player_info.theta;

            if moving{
                velocity.linvel = velocity.linvel + Vec3{ x: PLAYER_SPEED*input_theta.cos()*speed, y: 0.0, z: -PLAYER_SPEED*input_theta.sin()*speed };                        
            }

            if keyboard_input.pressed(KeyCode::Space) && player_info.objects_hit >=1{
                velocity.linvel.y = 10.0;
                player_info.has_hit_object = false;
            
            }

            // Fast fall

            if keyboard_input.pressed(KeyCode::LShift) && player_info.objects_hit <=1{
                velocity.linvel.y = -30.0;
            }

            // Dashing
            if moving && (keyboard_input.just_pressed(KeyCode::Q) || dashing){
                velocity.linvel = velocity.linvel + Vec3{ x: PLAYER_SPEED*input_theta.cos()*speed * 30.0, y: 0.0, z: -PLAYER_SPEED*input_theta.sin()*speed  * 30.0};
            }

            for ev in motion_evr.iter() {
                player_info.theta -= 0.001 * ev.delta.x;
                if ev.delta.y < 0.0 && player_info.v_theta <= 0.45*std::f32::consts::PI {
                    player_info.v_theta -= 0.01 * ev.delta.y;
                }

                if ev.delta.y > 0.0 && player_info.v_theta >= 0.05*std::f32::consts::PI {
                    player_info.v_theta -= 0.01 * ev.delta.y;
                }
            }


            // Make sure that v_theta cant go below 0 or above pi/2
            if player_info.v_theta < 0.05*std::f32::consts::PI{
                player_info.v_theta = 0.05*std::f32::consts::PI;
            }

            if player_info.v_theta > 0.45*std::f32::consts::PI{
                player_info.v_theta = 0.45*std::f32::consts::PI;
            }

            println!("v_theta is {}",player_info.v_theta);
   
        }

    }

    pub(crate) struct MyGamepad(Gamepad);

    pub(crate) fn gamepad_connections(
        mut commands: Commands,
        my_gamepad: Option<Res<MyGamepad>>,
        mut gamepad_evr: EventReader<GamepadEvent>,
    ) {
        for ev in gamepad_evr.iter() {
            // the ID of the gamepad
            let id = ev.gamepad;
            match ev.event_type {
                GamepadEventType::Connected => {
                    println!("New gamepad connected with ID: {:?}", id);

                    // if we don't have any gamepad yet, use this one
                    if my_gamepad.is_none() {
                        commands.insert_resource(MyGamepad(id));
                    }
                }
                GamepadEventType::Disconnected => {
                    println!("Lost gamepad connection with ID: {:?}", id);

                    // if it's the one we previously associated with the player,
                    // disassociate it:
                    if let Some(MyGamepad(old_id)) = my_gamepad.as_deref() {
                        if *old_id == id {
                            commands.remove_resource::<MyGamepad>();
                        }
                    }
                }
                // other events are irrelevant
                _ => {}
            }
        }
    }

}
