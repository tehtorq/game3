use glam::Vec3;
use crate::renderer::Renderer;
use crate::player::Player;
use crate::game::Game;

pub struct HUD {
    minimap_size: f32,
    minimap_range: f32,
}

impl HUD {
    pub fn new() -> Self {
        Self {
            minimap_size: 150.0,  // Size of minimap in pixels
            minimap_range: 15000.0, // Even more zoomed out for aggressive enemies
        }
    }
    
    pub fn zoom_in(&mut self) {
        self.minimap_range = (self.minimap_range * 0.75).max(500.0);
    }
    
    pub fn zoom_out(&mut self) {
        self.minimap_range = (self.minimap_range * 1.33).min(50000.0); // Allow zooming out to see entire map
    }
    
    pub fn draw(&self, renderer: &mut Renderer, game: &Game, screen_width: f32, screen_height: f32) {
        let player = &game.player;
        // Speed indicator (bottom left)
        let speed = player.get_speed();
        let speed_text = format!("SPEED: {:.0}", speed);
        self.draw_text(renderer, &speed_text, 20.0, screen_height - 40.0);
        
        // Altitude indicators (bottom left, below speed)
        let altitude = player.get_altitude();
        let terrain_height = player.get_height_above_terrain();
        
        let alt_text = format!("ALT: {:.0}", altitude);
        let terrain_text = format!("AGL: {:.0}", terrain_height); // Above Ground Level
        
        self.draw_text(renderer, &alt_text, 20.0, screen_height - 60.0);
        self.draw_text(renderer, &terrain_text, 20.0, screen_height - 80.0);
        
        // Afterburner fuel (bottom left, below altitude)
        let fuel_percent = (player.afterburner_fuel * 100.0) as i32;
        let fuel_text = format!("FUEL: {}%", fuel_percent);
        self.draw_text(renderer, &fuel_text, 20.0, screen_height - 100.0);
        
        // Shield indicator (bottom left, below fuel)
        let shield_percent = (player.shield * 100.0) as i32;
        let shield_text = format!("SHIELD: {}%", shield_percent);
        self.draw_text(renderer, &shield_text, 20.0, screen_height - 120.0);
        
        // Draw shield bar
        self.draw_shield_bar(renderer, player, 20.0, screen_height - 140.0);
        
        // Brake indicator (center bottom)
        if player.braking {
            self.draw_text(renderer, "BRAKING", screen_width / 2.0 - 40.0, screen_height - 40.0);
        }
        
        // Thrust vector indicator (shows direction of movement)
        self.draw_velocity_indicator(renderer, player, screen_width / 2.0, screen_height - 100.0);
        
        // Draw minimap (top right corner)
        self.draw_minimap(renderer, game, screen_width, screen_height);
        
        // Draw off-screen enemy indicators
        self.draw_offscreen_indicators(renderer, game, screen_width, screen_height);
    }
    
    fn draw_text(&self, renderer: &mut Renderer, text: &str, _x: f32, y: f32) {
        // For orthographic projection, use simpler coordinates
        let world_x = -35.0;
        let world_y = -35.0 + (600.0 - y) / 15.0; // Bottom-left origin
        let z = -1.0;
        
        // Draw a simple line to indicate text position
        let text_width = text.len() as f32 * 0.2;
        renderer.draw_line(
            Vec3::new(world_x, world_y, z),
            Vec3::new(world_x + text_width, world_y, z)
        );
    }
    
    fn draw_velocity_indicator(&self, renderer: &mut Renderer, player: &Player, _center_x: f32, _center_y: f32) {
        // Draw velocity indicator at center bottom
        let world_x = 0.0;
        let world_y = -30.0;
        let z = -1.0;
        
        let vel_normalized = player.vel.normalize_or_zero();
        if vel_normalized.length() > 0.0 {
            let scale = 2.0;
            
            // Draw velocity direction
            renderer.draw_line(
                Vec3::new(world_x, world_y, z),
                Vec3::new(world_x + vel_normalized.x * scale, world_y + vel_normalized.y * scale, z)
            );
        }
        
        // Always draw crosshair
        renderer.draw_line(
            Vec3::new(world_x - 0.5, world_y, z),
            Vec3::new(world_x + 0.5, world_y, z)
        );
        renderer.draw_line(
            Vec3::new(world_x, world_y - 0.5, z),
            Vec3::new(world_x, world_y + 0.5, z)
        );
    }
    
    fn draw_minimap(&self, renderer: &mut Renderer, game: &Game, screen_width: f32, screen_height: f32) {
        // Calculate aspect ratio to position correctly
        let aspect = screen_width / screen_height;
        
        // Position minimap in actual top-right corner
        // Right edge is at aspect * 40, top edge is at 40
        let half_size = 8.0; // Size in world units
        let margin = 2.0; // Distance from edge
        let map_center_x = aspect * 40.0 - half_size - margin;
        let map_center_y = 40.0 - half_size - margin;
        let z = -1.0;
        let corners = [
            Vec3::new(map_center_x - half_size, map_center_y - half_size, z),
            Vec3::new(map_center_x + half_size, map_center_y - half_size, z),
            Vec3::new(map_center_x + half_size, map_center_y + half_size, z),
            Vec3::new(map_center_x - half_size, map_center_y + half_size, z),
        ];
        
        // Draw border with double lines for visibility
        for i in 0..4 {
            let next = (i + 1) % 4;
            renderer.draw_line(corners[i], corners[next]);
        }
        
        // Draw grid lines for better visibility
        let grid_lines = 3;
        for i in 1..grid_lines {
            let offset = (i as f32 / grid_lines as f32 - 0.5) * 2.0 * half_size;
            // Vertical lines
            renderer.draw_line(
                Vec3::new(map_center_x + offset, map_center_y - half_size, z),
                Vec3::new(map_center_x + offset, map_center_y + half_size, z)
            );
            // Horizontal lines
            renderer.draw_line(
                Vec3::new(map_center_x - half_size, map_center_y + offset, z),
                Vec3::new(map_center_x + half_size, map_center_y + offset, z)
            );
        }
        
        // Draw compass directions (rotated based on player direction)
        // North is along -Z in world space
        // Transform it to view space
        let north_world = Vec3::new(0.0, 0.0, -1.0);
        let sin_a = game.player.rotation.sin();
        let cos_a = game.player.rotation.cos();
        let north_right = north_world.x * (-cos_a) + north_world.z * sin_a;
        let north_forward = north_world.x * sin_a + north_world.z * cos_a;
        let north_x = north_right * (half_size - 1.0);
        let north_y = -north_forward * (half_size - 1.0);
        renderer.draw_line(
            Vec3::new(map_center_x + north_x, map_center_y + north_y, z),
            Vec3::new(map_center_x + north_x * 1.2, map_center_y + north_y * 1.2, z)
        );
        // Draw 'N' indicator
        let n_size = 0.3;
        let n_pos = Vec3::new(map_center_x + north_x * 1.3, map_center_y + north_y * 1.3, z);
        renderer.draw_line(n_pos + Vec3::new(-n_size, -n_size, 0.0), n_pos + Vec3::new(-n_size, n_size, 0.0));
        renderer.draw_line(n_pos + Vec3::new(-n_size, n_size, 0.0), n_pos + Vec3::new(n_size, -n_size, 0.0));
        renderer.draw_line(n_pos + Vec3::new(n_size, -n_size, 0.0), n_pos + Vec3::new(n_size, n_size, 0.0));
        
        // Player position is always at center, always pointing up
        let player_map_pos = Vec3::new(map_center_x, map_center_y, z);
        
        // Draw player as a small triangle always pointing up
        let player_size = 0.5;
        let front = Vec3::new(0.0, player_size, 0.0);
        let left = Vec3::new(-player_size * 0.5, -player_size * 0.5, 0.0);
        let right = Vec3::new(player_size * 0.5, -player_size * 0.5, 0.0);
        
        renderer.draw_line(player_map_pos + front, player_map_pos + left);
        renderer.draw_line(player_map_pos + left, player_map_pos + right);
        renderer.draw_line(player_map_pos + right, player_map_pos + front);
        
        // Draw bases on minimap
        for base in &game.bases {
            if base.is_active {
                let relative_pos = base.pos - game.player.pos;
                let distance = relative_pos.length();
                
                // Always show bases on minimap (they're important landmarks)
                if distance < self.minimap_range * 2.0 {
                    // Transform to view space
                    let sin_a = game.player.rotation.sin();
                    let cos_a = game.player.rotation.cos();
                    
                    let view_right = relative_pos.x * (-cos_a) + relative_pos.z * sin_a;
                    let view_forward = relative_pos.x * sin_a + relative_pos.z * cos_a;
                    
                    // Scale position to minimap
                    let scale = half_size / self.minimap_range;
                    let base_x = map_center_x + view_right * scale;
                    let base_y = map_center_y - view_forward * scale;
                    
                    // Clamp to minimap bounds
                    let base_x = base_x.clamp(map_center_x - half_size, map_center_x + half_size);
                    let base_y = base_y.clamp(map_center_y - half_size, map_center_y + half_size);
                    
                    let base_map_pos = Vec3::new(base_x, base_y, z);
                    
                    // Draw base as square with size based on type
                    let base_size = match base.base_type {
                        crate::base::BaseType::Small => 0.3,
                        crate::base::BaseType::Medium => 0.4,
                        crate::base::BaseType::Large => 0.5,
                        crate::base::BaseType::Fortress => 0.6,
                    };
                    
                    // Draw square outline
                    renderer.draw_line(
                        base_map_pos + Vec3::new(-base_size, -base_size, 0.0),
                        base_map_pos + Vec3::new(base_size, -base_size, 0.0)
                    );
                    renderer.draw_line(
                        base_map_pos + Vec3::new(base_size, -base_size, 0.0),
                        base_map_pos + Vec3::new(base_size, base_size, 0.0)
                    );
                    renderer.draw_line(
                        base_map_pos + Vec3::new(base_size, base_size, 0.0),
                        base_map_pos + Vec3::new(-base_size, base_size, 0.0)
                    );
                    renderer.draw_line(
                        base_map_pos + Vec3::new(-base_size, base_size, 0.0),
                        base_map_pos + Vec3::new(-base_size, -base_size, 0.0)
                    );
                    
                    // Draw center cross to indicate it's a base
                    renderer.draw_line(
                        base_map_pos + Vec3::new(-base_size * 0.5, 0.0, 0.0),
                        base_map_pos + Vec3::new(base_size * 0.5, 0.0, 0.0)
                    );
                    renderer.draw_line(
                        base_map_pos + Vec3::new(0.0, -base_size * 0.5, 0.0),
                        base_map_pos + Vec3::new(0.0, base_size * 0.5, 0.0)
                    );
                }
            }
        }
        
        // Draw enemies on minimap
        for enemy in &game.enemies {
            let relative_pos = enemy.pos - game.player.pos;
            let distance = relative_pos.length();
            
            // Only show enemies within minimap range
            if distance < self.minimap_range {
                // Rotate relative position to align with player's view
                // Player forward is (-sin(rot), 0, -cos(rot))
                // Right is (-cos(rot), 0, sin(rot))
                // We want to transform world coordinates to view coordinates
                let sin_a = game.player.rotation.sin();
                let cos_a = game.player.rotation.cos();
                
                // Transform to view space where forward is up on minimap
                let view_right = relative_pos.x * (-cos_a) + relative_pos.z * sin_a;
                let view_forward = relative_pos.x * sin_a + relative_pos.z * cos_a;
                
                // Scale position to minimap
                let scale = half_size / self.minimap_range;
                let enemy_x = map_center_x + view_right * scale;
                let enemy_y = map_center_y - view_forward * scale; // Negative because forward should be up
                
                // Clamp to minimap bounds
                let enemy_x = enemy_x.clamp(map_center_x - half_size, map_center_x + half_size);
                let enemy_y = enemy_y.clamp(map_center_y - half_size, map_center_y + half_size);
                
                let enemy_map_pos = Vec3::new(enemy_x, enemy_y, z);
                
                // Draw enemy as small diamond
                let size = 0.15;
                renderer.draw_line(
                    enemy_map_pos + Vec3::new(0.0, -size, 0.0),
                    enemy_map_pos + Vec3::new(size, 0.0, 0.0)
                );
                renderer.draw_line(
                    enemy_map_pos + Vec3::new(size, 0.0, 0.0),
                    enemy_map_pos + Vec3::new(0.0, size, 0.0)
                );
                renderer.draw_line(
                    enemy_map_pos + Vec3::new(0.0, size, 0.0),
                    enemy_map_pos + Vec3::new(-size, 0.0, 0.0)
                );
                renderer.draw_line(
                    enemy_map_pos + Vec3::new(-size, 0.0, 0.0),
                    enemy_map_pos + Vec3::new(0.0, -size, 0.0)
                );
            }
        }
        
        // Draw terrain elevation hints
        let terrain_samples = 5;
        let angle = game.player.rotation;
        for i in 0..terrain_samples {
            for j in 0..terrain_samples {
                let sample_x = (i as f32 / (terrain_samples - 1) as f32 - 0.5) * 2.0 * self.minimap_range;
                let sample_z = (j as f32 / (terrain_samples - 1) as f32 - 0.5) * 2.0 * self.minimap_range;
                
                // Rotate sample position to world space (inverse rotation)
                let world_offset_x = sample_x * angle.cos() - sample_z * angle.sin();
                let world_offset_z = sample_x * angle.sin() + sample_z * angle.cos();
                
                let world_x = game.player.pos.x + world_offset_x;
                let world_z = game.player.pos.z + world_offset_z;
                
                let terrain_height = terrain_height_at(world_x, world_z);
                let height_diff = terrain_height - game.player.pos.y;
                
                // Only show significant elevation differences
                if height_diff.abs() > 50.0 {
                    // Map position (already rotated)
                    let map_x = map_center_x + sample_x * half_size / self.minimap_range;
                    let map_y = map_center_y - sample_z * half_size / self.minimap_range;
                    
                    if map_x >= map_center_x - half_size && map_x <= map_center_x + half_size &&
                       map_y >= map_center_y - half_size && map_y <= map_center_y + half_size {
                        // Draw elevation marker
                        let marker_size = 0.1;
                        if height_diff > 0.0 {
                            // Higher terrain - draw ^
                            renderer.draw_line(
                                Vec3::new(map_x - marker_size, map_y + marker_size, z),
                                Vec3::new(map_x, map_y - marker_size, z)
                            );
                            renderer.draw_line(
                                Vec3::new(map_x, map_y - marker_size, z),
                                Vec3::new(map_x + marker_size, map_y + marker_size, z)
                            );
                        } else {
                            // Lower terrain - draw v
                            renderer.draw_line(
                                Vec3::new(map_x - marker_size, map_y - marker_size, z),
                                Vec3::new(map_x, map_y + marker_size, z)
                            );
                            renderer.draw_line(
                                Vec3::new(map_x, map_y + marker_size, z),
                                Vec3::new(map_x + marker_size, map_y - marker_size, z)
                            );
                        }
                    }
                }
            }
        }
    }
    
    fn draw_offscreen_indicators(&self, renderer: &mut Renderer, game: &Game, screen_width: f32, screen_height: f32) {
        // Calculate screen bounds with some margin
        let margin = 50.0;
        let z = -50.0; // HUD depth
        
        for enemy in &game.enemies {
            let to_enemy = enemy.pos - game.player.pos;
            let distance = to_enemy.length();
            
            // Only show indicators for enemies within a reasonable range
            if distance < 2000.0 && distance > 50.0 {
                // Project enemy position to screen space (simplified)
                // In a real implementation, this would use proper view/projection matrices
                let player_forward = Vec3::new(-game.player.rotation.sin(), 0.0, -game.player.rotation.cos());
                let player_right = Vec3::new(-game.player.rotation.cos(), 0.0, game.player.rotation.sin());
                
                // Get enemy position relative to player's view
                let forward_dist = to_enemy.dot(player_forward);
                let right_dist = to_enemy.dot(player_right);
                let up_dist = to_enemy.y;
                
                // Simple perspective projection
                if forward_dist > 10.0 { // Enemy is in front
                    let screen_x = screen_width / 2.0 + (right_dist / forward_dist) * 300.0;
                    let screen_y = screen_height / 2.0 - (up_dist / forward_dist) * 300.0;
                    
                    // Check if enemy is off-screen
                    if screen_x < margin || screen_x > screen_width - margin || 
                       screen_y < margin || screen_y > screen_height - margin {
                        // Calculate edge position for indicator
                        let center_x = screen_width / 2.0;
                        let center_y = screen_height / 2.0;
                        
                        let dx = screen_x - center_x;
                        let dy = screen_y - center_y;
                        let angle = dy.atan2(dx);
                        
                        // Find intersection with screen edge
                        let edge_x: f32;
                        let edge_y: f32;
                        
                        let max_x = (screen_width / 2.0 - margin) / dx.abs();
                        let max_y = (screen_height / 2.0 - margin) / dy.abs();
                        let scale = max_x.min(max_y);
                        
                        if scale < 1.0 {
                            edge_x = center_x + dx * scale;
                            edge_y = center_y + dy * scale;
                        } else {
                            continue; // Enemy is on screen
                        }
                        
                        // Convert to world coordinates for rendering
                        let world_x = (edge_x - 400.0) * 0.1;
                        let world_y = (edge_y - 300.0) * 0.1;
                        
                        // Draw arrow pointing towards enemy
                        let arrow_size = 0.5;
                        let arrow_pos = Vec3::new(world_x, world_y, z);
                        
                        // Arrow pointing inward
                        let arrow_dir = Vec3::new(-angle.cos(), -angle.sin(), 0.0);
                        let arrow_perp = Vec3::new(angle.sin(), -angle.cos(), 0.0);
                        
                        renderer.draw_line(
                            arrow_pos,
                            arrow_pos + arrow_dir * arrow_size
                        );
                        renderer.draw_line(
                            arrow_pos,
                            arrow_pos + arrow_dir * arrow_size * 0.5 + arrow_perp * arrow_size * 0.3
                        );
                        renderer.draw_line(
                            arrow_pos,
                            arrow_pos + arrow_dir * arrow_size * 0.5 - arrow_perp * arrow_size * 0.3
                        );
                        
                        // Draw distance text (simplified - just a small line indicating distance)
                        let dist_indicator_length = (2000.0 - distance) / 2000.0 * 0.5;
                        renderer.draw_line(
                            arrow_pos + arrow_dir * arrow_size * 1.2,
                            arrow_pos + arrow_dir * (arrow_size * 1.2 + dist_indicator_length)
                        );
                    }
                }
            }
        }
    }
    
    fn draw_shield_bar(&self, renderer: &mut Renderer, player: &Player, x: f32, y: f32) {
        // Draw shield strength as a horizontal bar
        let world_x = -35.0 + (x / 100.0);
        let world_y = 25.0 - (y / 100.0);
        let z = -1.0;
        
        let bar_width = 5.0;
        let bar_height = 0.3;
        
        // Draw bar outline
        renderer.draw_line(
            Vec3::new(world_x, world_y, z),
            Vec3::new(world_x + bar_width, world_y, z)
        );
        renderer.draw_line(
            Vec3::new(world_x, world_y - bar_height, z),
            Vec3::new(world_x + bar_width, world_y - bar_height, z)
        );
        renderer.draw_line(
            Vec3::new(world_x, world_y, z),
            Vec3::new(world_x, world_y - bar_height, z)
        );
        renderer.draw_line(
            Vec3::new(world_x + bar_width, world_y, z),
            Vec3::new(world_x + bar_width, world_y - bar_height, z)
        );
        
        // Draw filled portion based on shield strength
        let fill_width = bar_width * player.shield;
        if fill_width > 0.1 {
            // Draw several horizontal lines to fill the bar
            for i in 1..4 {
                let offset = i as f32 * bar_height / 4.0;
                renderer.draw_line(
                    Vec3::new(world_x, world_y - offset, z),
                    Vec3::new(world_x + fill_width, world_y - offset, z)
                );
            }
        }
        
        // If shield is recharging, draw a pulsing indicator
        if player.shield_recharge_timer > 0.0 {
            let pulse = (player.shield_recharge_timer * 3.0).sin().abs();
            renderer.draw_line(
                Vec3::new(world_x - 0.2, world_y - bar_height * 0.5, z),
                Vec3::new(world_x - 0.2 - pulse * 0.3, world_y - bar_height * 0.5, z)
            );
        }
    }
}

// Calculate terrain height at a given position (matches shader calculation)
fn terrain_height_at(x: f32, z: f32) -> f32 {
    let base_y = 20.0;
    
    // Large-scale terrain features - very broad valleys and mountains (4x amplified)
    let mut large_scale = (x * 0.0005).sin() * (z * 0.0007).sin() * 240.0;
    large_scale += (x * 0.0003 + 1.5).cos() * (z * 0.0004 - 0.8).sin() * 200.0;
    
    // Create base terrain with gentle slopes
    let mut gentle = (x * 0.0031).sin() * (z * 0.0027).cos() * 25.0;
    gentle += (x * 0.0047).sin() * (z * 0.0053).sin() * 20.0;
    
    // Create a "roughness map" that determines where bumpy areas appear
    let mut roughness = (x * 0.0023 + 2.7).sin() * (z * 0.0019 - 1.3).cos();
    roughness += (x * 0.0041 - z * 0.0037).sin() * 0.5;
    roughness = (roughness + 1.5) / 3.0; // Normalize to ~0-1 range
    
    // Make roughness more sparse by thresholding (approximating smoothstep)
    roughness = if roughness < 0.6 { 0.0 } else if roughness > 0.8 { 1.0 } else { (roughness - 0.6) / 0.2 };
    
    // Bumpy terrain details
    let mut bumps = 0.0;
    bumps += (x * 0.0173).sin() * (z * 0.0199).sin() * 20.0;
    bumps += (x * 0.0293 + 2.1).cos() * (z * 0.0311 - 1.7).sin() * 15.0;
    bumps += (x * 0.0519 + z * 0.0413).sin() * 8.0;
    bumps += (x * 0.0871 - z * 0.0926).sin() * 5.0;
    bumps += (x * 0.137).sin() * (z * 0.149).cos() * 3.0;
    
    // Combine all terrain features
    base_y + large_scale + gentle + (bumps * roughness)
}