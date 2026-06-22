use crate::engine::{FilterEngine, FilterUniforms};
use crate::types::{FilterError, FilterResult};

impl FilterEngine {
    pub(crate) fn evaluate_primitive(
        &mut self,
        kind: &usvg::filter::Kind,
        input_views: &[std::sync::Arc<wgpu::TextureView>],
        rect: usvg::NonZeroRect,
        element_bbox: usvg::NonZeroRect,
        color_interpolation: usvg::filter::ColorInterpolation,
    ) -> Result<FilterResult, FilterError> {
        self.color_interpolation = color_interpolation;
        let w = rect.width().ceil().max(1.0) as u32;
        let h = rect.height().ceil().max(1.0) as u32;

        match kind {
            usvg::filter::Kind::GaussianBlur(gb) => {
                self.apply_gaussian_blur(&*input_views[0], w, h, gb)
            }
            usvg::filter::Kind::ColorMatrix(cm) => {
                self.apply_color_matrix(&*input_views[0], w, h, cm)
            }
            usvg::filter::Kind::Blend(blend) => {
                self.apply_blend(&*input_views[0], &*input_views[1], w, h, blend)
            }
            usvg::filter::Kind::Composite(comp) => {
                self.apply_composite(&*input_views[0], &*input_views[1], w, h, comp)
            }
            usvg::filter::Kind::Flood(flood) => self.apply_flood(w, h, flood),
            usvg::filter::Kind::Offset(offset) => {
                self.apply_offset(&*input_views[0], w, h, rect, element_bbox, offset)
            }
            usvg::filter::Kind::Merge(merge) => {
                self.apply_merge(input_views, w, h, merge)
            }
            usvg::filter::Kind::DropShadow(ds) => self.apply_drop_shadow(&*input_views[0], w, h, ds),
            usvg::filter::Kind::ComponentTransfer(ct) => {
                self.apply_component_transfer(&*input_views[0], w, h, ct)
            }
            usvg::filter::Kind::ConvolveMatrix(cm) => {
                self.apply_convolve_matrix(&*input_views[0], w, h, cm)
            }
            usvg::filter::Kind::DisplacementMap(dm) => {
                self.apply_displacement_map(&*input_views[0], &*input_views[1], w, h, dm)
            }
            usvg::filter::Kind::Morphology(m) => self.apply_morphology(&*input_views[0], w, h, m),
            usvg::filter::Kind::Tile(tile) => {
                self.apply_tile(&*input_views[0], w, h, rect, element_bbox, tile)
            }
            usvg::filter::Kind::Turbulence(t) => self.apply_turbulence(w, h, t),
            usvg::filter::Kind::DiffuseLighting(dl) => {
                self.apply_diffuse_lighting(&*input_views[0], w, h, dl)
            }
            usvg::filter::Kind::SpecularLighting(sl) => {
                self.apply_specular_lighting(&*input_views[0], w, h, sl)
            }
            usvg::filter::Kind::Image(img) => self.apply_image(&*input_views[0], w, h, img),
        }
    }

    // ── Gaussian Blur (Two-Pass Separable) ──────────────────────────────────

    fn apply_gaussian_blur(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        gb: &usvg::filter::GaussianBlur,
    ) -> Result<FilterResult, FilterError> {
        let std_x = gb.std_dev_x().get();
        let std_y = gb.std_dev_y().get();
        let radius_x = ((std_x * 3.0).ceil() as u32).min(64);
        let radius_y = ((std_y * 3.0).ceil() as u32).min(64);

        if radius_x == 0 && radius_y == 0 {
            return self.apply_passthrough(input, w, h);
        }

        let input_size = (w as f32, h as f32);
        let temp_view = self.get_temp_view(w, h)?;
        let output_view = self.get_temp_view(w, h)?;
        let sampler = &self.linear_sampler;

        // Horizontal pass.
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.param0 = radius_x as f32;
        params.param1 = std_x;
        self.render_pass(
            input,
            sampler,
            input_size,
            &temp_view,
            0, // MODE_GAUSSIAN_BLUR_H
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        // Vertical pass.
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.param0 = radius_y as f32;
        params.param1 = std_y;
        self.render_pass(
            &temp_view,
            sampler,
            input_size,
            &output_view,
            1, // MODE_GAUSSIAN_BLUR_V
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Color Matrix ────────────────────────────────────────────────────────

    fn apply_color_matrix(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        cm: &usvg::filter::ColorMatrix,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let values: &[f32] = match cm.kind() {
            usvg::filter::ColorMatrixKind::Matrix(v) => v.as_slice(),
            _ => &[
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
            ],
        };
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.cm_row0 = [values[0], values[1], values[2], values[3]];
        params.cm_row1 = [values[5], values[6], values[7], values[8]];
        params.cm_row2 = [values[10], values[11], values[12], values[13]];
        params.cm_row3 = [values[15], values[16], values[17], values[18]];
        params.param0 = values[4]; // offset r
        params.param1 = values[9]; // offset g
        params.param2 = values[14]; // offset b
        params.param3 = values[19]; // offset a

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            2, // MODE_COLOR_MATRIX
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Blend ───────────────────────────────────────────────────────────────

    fn apply_blend(
        &mut self,
        input_a: &wgpu::TextureView,
        input_b: &wgpu::TextureView,
        w: u32,
        h: u32,
        blend: &usvg::filter::Blend,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let sub_mode = match blend.mode() {
            usvg::BlendMode::Normal => 0u32,
            usvg::BlendMode::Multiply => 1u32,
            usvg::BlendMode::Screen => 2u32,
            usvg::BlendMode::Darken => 3u32,
            usvg::BlendMode::Lighten => 4u32,
            usvg::BlendMode::Overlay => 5u32,
            usvg::BlendMode::HardLight => 6u32,
            usvg::BlendMode::SoftLight => 7u32,
            usvg::BlendMode::ColorDodge => 8u32,
            usvg::BlendMode::ColorBurn => 9u32,
            usvg::BlendMode::Exclusion => 10u32,
            usvg::BlendMode::Hue => 11u32,
            usvg::BlendMode::Saturation => 12u32,
            usvg::BlendMode::Color => 13u32,
            usvg::BlendMode::Luminosity => 14u32,
            usvg::BlendMode::Difference => 0u32,
        };

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];

        self.render_pass(
            input_a,
            &self.linear_sampler,
            input_size,
            &output_view,
            3, // MODE_BLEND
            sub_mode,
            &params,
            Some(input_b),
            Some(&self.linear_sampler),
            input_size,
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Composite ───────────────────────────────────────────────────────────

    fn apply_composite(
        &mut self,
        input_a: &wgpu::TextureView,
        input_b: &wgpu::TextureView,
        w: u32,
        h: u32,
        comp: &usvg::filter::Composite,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];

        let sub_mode = match comp.operator() {
            usvg::filter::CompositeOperator::Over => 0u32,
            usvg::filter::CompositeOperator::In => 1u32,
            usvg::filter::CompositeOperator::Out => 2u32,
            usvg::filter::CompositeOperator::Atop => 3u32,
            usvg::filter::CompositeOperator::Xor => 4u32,
            usvg::filter::CompositeOperator::Arithmetic { k1, k2, k3, k4 } => {
                params.param0 = k1;
                params.param1 = k2;
                params.param2 = k3;
                params.param3 = k4;
                6u32
            }
        };

        self.render_pass(
            input_a,
            &self.linear_sampler,
            input_size,
            &output_view,
            4, // MODE_COMPOSITE
            sub_mode,
            &params,
            Some(input_b),
            Some(&self.linear_sampler),
            input_size,
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Flood ───────────────────────────────────────────────────────────────

    fn apply_flood(
        &mut self,
        w: u32,
        h: u32,
        flood: &usvg::filter::Flood,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;

        let c = flood.color();
        let o = flood.opacity().get();
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.flood_color = [
            c.red as f32 / 255.0,
            c.green as f32 / 255.0,
            c.blue as f32 / 255.0,
            o,
        ];

        self.render_pass(
            &output_view,
            &self.nearest_sampler,
            (w as f32, h as f32),
            &output_view,
            5, // MODE_FLOOD
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Offset ──────────────────────────────────────────────────────────────

    fn apply_offset(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        rect: usvg::NonZeroRect,
        _element_bbox: usvg::NonZeroRect,
        offset: &usvg::filter::Offset,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let (dx, dy) = (
            offset.dx() / rect.width().max(0.001),
            offset.dy() / rect.height().max(0.001),
        );

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.offset = [dx, dy];

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            6, // MODE_OFFSET
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Merge ───────────────────────────────────────────────────────────────

    fn apply_merge(
        &mut self,
        inputs: &[std::sync::Arc<wgpu::TextureView>],
        w: u32,
        h: u32,
        _merge: &usvg::filter::Merge,
    ) -> Result<FilterResult, FilterError> {
        if inputs.is_empty() {
            return Err(FilterError::UnresolvedInput("merge: no inputs".into()));
        }

        let input_size = (w as f32, h as f32);

        let mut result_view = inputs[0].clone();
        for input in inputs.iter().skip(1) {
            let temp_out = self.get_temp_view(w, h)?;
            let mut params = FilterUniforms::default();
            params.region = [0.0, 0.0, w as f32, h as f32];
            self.render_pass(
                &result_view,
                &self.linear_sampler,
                input_size,
                &temp_out,
                7, // MODE_MERGE
                0,
                &params,
                Some(input),
                Some(&self.linear_sampler),
                input_size,
            )?;
            result_view = std::sync::Arc::new(temp_out);
        }

        Ok(FilterResult {
            output_view: result_view,
            region: (0, 0, w, h),
        })
    }

    // ── Drop Shadow ─────────────────────────────────────────────────────────

    fn apply_drop_shadow(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        ds: &usvg::filter::DropShadow,
    ) -> Result<FilterResult, FilterError> {
        let std_x = ds.std_dev_x().get();
        let std_y = ds.std_dev_y().get();
        let radius_x = ((std_x * 3.0).ceil() as u32).min(64);
        let radius_y = ((std_y * 3.0).ceil() as u32).min(64);
        let input_size = (w as f32, h as f32);

        let c = ds.color();
        let o = ds.opacity().get();
        let flood_view = self.get_temp_view(w, h)?;
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.flood_color = [
            c.red as f32 / 255.0,
            c.green as f32 / 255.0,
            c.blue as f32 / 255.0,
            o,
        ];
        self.render_pass(
            &flood_view,
            &self.nearest_sampler,
            input_size,
            &flood_view,
            5,
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        if radius_x > 0 || radius_y > 0 {
            let blur_temp = self.get_temp_view(w, h)?;
            let mut blur_params = FilterUniforms::default();
            blur_params.region = [0.0, 0.0, w as f32, h as f32];
            blur_params.param0 = radius_x as f32;
            blur_params.param1 = std_x;
            self.render_pass(
                &flood_view,
                &self.linear_sampler,
                input_size,
                &blur_temp,
                0,
                0,
                &blur_params,
                None,
                None,
                (0.0, 0.0),
            )?;
            let mut blur_params2 = FilterUniforms::default();
            blur_params2.region = [0.0, 0.0, w as f32, h as f32];
            blur_params2.param0 = radius_y as f32;
            blur_params2.param1 = std_y;
            self.render_pass(
                &blur_temp,
                &self.linear_sampler,
                input_size,
                &flood_view,
                1,
                0,
                &blur_params2,
                None,
                None,
                (0.0, 0.0),
            )?;
        }

        let offset_view = self.get_temp_view(w, h)?;
        let mut offset_params = FilterUniforms::default();
        offset_params.region = [0.0, 0.0, w as f32, h as f32];
        offset_params.offset = [ds.dx() / w as f32, ds.dy() / h as f32];
        self.render_pass(
            &flood_view,
            &self.linear_sampler,
            input_size,
            &offset_view,
            6,
            0,
            &offset_params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_view = self.get_temp_view(w, h)?;
        let mut merge_params = FilterUniforms::default();
        merge_params.region = [0.0, 0.0, w as f32, h as f32];
        self.render_pass(
            &offset_view,
            &self.linear_sampler,
            input_size,
            &output_view,
            7,
            0,
            &merge_params,
            Some(input),
            Some(&self.linear_sampler),
            input_size,
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Component Transfer ──────────────────────────────────────────────────

    fn apply_component_transfer(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        ct: &usvg::filter::ComponentTransfer,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let func = ct.func_r();
        let sub_mode = match func {
            usvg::filter::TransferFunction::Identity => 0u32,
            usvg::filter::TransferFunction::Table(_) => 1u32,
            usvg::filter::TransferFunction::Discrete(_) => 2u32,
            usvg::filter::TransferFunction::Linear { .. } => 3u32,
            usvg::filter::TransferFunction::Gamma { .. } => 4u32,
        };

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        match func {
            usvg::filter::TransferFunction::Linear { slope, intercept } => {
                params.param1 = *slope;
                params.param2 = *intercept;
            }
            usvg::filter::TransferFunction::Gamma {
                amplitude,
                exponent,
                offset,
            } => {
                params.param0 = *amplitude;
                params.param1 = *exponent;
                params.param2 = *offset;
            }
            _ => {}
        }

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            8, // MODE_COMPONENT_XFER
            sub_mode,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    fn apply_convolve_matrix(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        cm: &usvg::filter::ConvolveMatrix,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let matrix = cm.matrix();
        let values = matrix.data();
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        if values.len() >= 9 {
            params.kernel = [values[0], values[1], values[2], values[3]];
            params.kernel2 = [values[4], values[5], values[6], values[7]];
            params.kernel3 = values[8];
        }
        params.kernel_divisor = cm.divisor().get();
        params.kernel_bias = cm.bias();

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            9, // MODE_CONVOLVE
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Displacement Map ────────────────────────────────────────────────────

    fn apply_displacement_map(
        &mut self,
        input: &wgpu::TextureView,
        displacement: &wgpu::TextureView,
        w: u32,
        h: u32,
        dm: &usvg::filter::DisplacementMap,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.disp_scale = dm.scale();
        let x_sel = match dm.x_channel_selector() {
            usvg::filter::ColorChannel::R => 0u32,
            usvg::filter::ColorChannel::G => 1u32,
            usvg::filter::ColorChannel::B => 2u32,
            usvg::filter::ColorChannel::A => 3u32,
        };
        let y_sel = match dm.y_channel_selector() {
            usvg::filter::ColorChannel::R => 0u32,
            usvg::filter::ColorChannel::G => 1u32,
            usvg::filter::ColorChannel::B => 2u32,
            usvg::filter::ColorChannel::A => 3u32,
        };
        params.sub_mode = x_sel | (y_sel << 2);

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            10, // MODE_DISPLACEMENT
            params.sub_mode,
            &params,
            Some(displacement),
            Some(&self.linear_sampler),
            input_size,
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Morphology ──────────────────────────────────────────────────────────

    fn apply_morphology(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        m: &usvg::filter::Morphology,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let rx = m.radius_x();
        let ry = m.radius_y();
        let sub_mode = match m.operator() {
            usvg::filter::MorphologyOperator::Erode => 0u32,
            usvg::filter::MorphologyOperator::Dilate => 1u32,
        };

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.param0 = rx.get();
        params.param1 = ry.get();

        self.render_pass(
            input,
            &self.nearest_sampler,
            input_size,
            &output_view,
            11, // MODE_MORPHOLOGY
            sub_mode,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Tile ────────────────────────────────────────────────────────────────

    fn apply_tile(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        rect: usvg::NonZeroRect,
        _element_bbox: usvg::NonZeroRect,
        _tile: &usvg::filter::Tile,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut params = FilterUniforms::default();
        params.region = [rect.x(), rect.y(), rect.width(), rect.height()];

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            12, // MODE_TILE
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Turbulence ──────────────────────────────────────────────────────────

    fn apply_turbulence(
        &mut self,
        w: u32,
        h: u32,
        t: &usvg::filter::Turbulence,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;

        let bfx = t.base_frequency_x();
        let bfy = t.base_frequency_y();
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.turb_base_freq = [bfx.get(), bfy.get()];
        params.turb_seed = t.seed() as f32;
        params.turb_num_octaves = t.num_octaves() as f32;

        self.render_pass(
            &output_view,
            &self.nearest_sampler,
            (w as f32, h as f32),
            &output_view,
            13, // MODE_TURBULENCE
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Normal Map ──────────────────────────────────────────────────────────────

    #[allow(dead_code)]
    fn apply_normal_map(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.light_surface_scale = 1.0;

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            14, // MODE_NORMAL_MAP
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Diffuse Lighting ───────────────────────────────────────────────────────

    fn apply_diffuse_lighting(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        dl: &usvg::filter::DiffuseLighting,
    ) -> Result<FilterResult, FilterError> {
        let normal_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut normal_params = FilterUniforms::default();
        normal_params.region = [0.0, 0.0, w as f32, h as f32];
        normal_params.light_surface_scale = dl.surface_scale();
        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &normal_view,
            14, // MODE_NORMAL_MAP
            0,
            &normal_params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_view = self.get_temp_view(w, h)?;

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.light_diffuse_k = dl.diffuse_constant();
        params.light_ambient = 0.0;
        params.light_color = [1.0, 1.0, 1.0];
        params.light_position = [0.5, 0.5, 1.0];
        params.light_surface_scale = dl.surface_scale();

        let light = dl.light_source();
        match light {
            usvg::filter::LightSource::DistantLight(dl_light) => {
                let azimuth = dl_light.azimuth.to_radians();
                let elevation = dl_light.elevation.to_radians();
                params.light_position = [
                    azimuth.cos() * elevation.cos(),
                    azimuth.sin() * elevation.cos(),
                    elevation.sin(),
                ];
            }
            usvg::filter::LightSource::PointLight(pl) => {
                params.light_position = [pl.x, pl.y, pl.z];
            }
            usvg::filter::LightSource::SpotLight(sl) => {
                params.light_position = [sl.x, sl.y, sl.z];
            }
        }

        self.render_pass(
            &normal_view,
            &self.linear_sampler,
            input_size,
            &output_view,
            15, // MODE_DIFFUSE_LIGHT
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Specular Lighting ──────────────────────────────────────────────────────

    fn apply_specular_lighting(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        sl: &usvg::filter::SpecularLighting,
    ) -> Result<FilterResult, FilterError> {
        let normal_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut normal_params = FilterUniforms::default();
        normal_params.region = [0.0, 0.0, w as f32, h as f32];
        normal_params.light_surface_scale = sl.surface_scale();
        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &normal_view,
            14, // MODE_NORMAL_MAP
            0,
            &normal_params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_view = self.get_temp_view(w, h)?;

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.light_specular_k = sl.specular_constant();
        params.light_shininess = sl.specular_exponent();
        params.light_ambient = 0.0;
        params.light_diffuse_k = 0.0;
        params.light_color = [1.0, 1.0, 1.0];
        params.light_position = [0.5, 0.5, 1.0];
        params.light_surface_scale = sl.surface_scale();

        let light = sl.light_source();
        match light {
            usvg::filter::LightSource::DistantLight(dl_light) => {
                let azimuth = dl_light.azimuth.to_radians();
                let elevation = dl_light.elevation.to_radians();
                params.light_position = [
                    azimuth.cos() * elevation.cos(),
                    azimuth.sin() * elevation.cos(),
                    elevation.sin(),
                ];
            }
            usvg::filter::LightSource::PointLight(pl) => {
                params.light_position = [pl.x, pl.y, pl.z];
            }
            usvg::filter::LightSource::SpotLight(sp) => {
                params.light_position = [sp.x, sp.y, sp.z];
            }
        }

        self.render_pass(
            &normal_view,
            &self.linear_sampler,
            input_size,
            &output_view,
            16, // MODE_SPECULAR_LIGHT
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Component Transfer LUT ─────────────────────────────────────────────────

    #[allow(dead_code)]
    fn apply_component_transfer_lut(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        _ct: &usvg::filter::ComponentTransfer,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            17, // MODE_COMPONENT_XFER_LUT
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Image ──────────────────────────────────────────────────────────────────

    fn apply_image(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        img: &usvg::filter::Image,
    ) -> Result<FilterResult, FilterError> {
        let root = img.root();
        if root.has_children() {
            let id = root.id();
            if !id.is_empty() {
                if let Some((_tex, view)) = self.image_textures.get(id) {
                    return Ok(FilterResult {
                        output_view: std::sync::Arc::new(view.clone()),
                        region: (0, 0, w, h),
                    });
                }
            }
        }
        self.apply_passthrough(input, w, h)
    }

    // ── Passthrough ─────────────────────────────────────────────────────────

    fn apply_passthrough(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
    ) -> Result<FilterResult, FilterError> {
        Ok(FilterResult {
            output_view: std::sync::Arc::new(input.clone()),
            region: (0, 0, w, h),
        })
    }
}
