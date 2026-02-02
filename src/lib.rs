use aviutl2::{
    anyhow::{self, Context},
    log,
};

#[aviutl2::plugin(GenericPlugin)]
#[derive(Debug)]
struct ClipboardAux {}

static EDIT_HANDLE: std::sync::OnceLock<aviutl2::generic::EditHandle> = std::sync::OnceLock::new();

impl aviutl2::generic::GenericPlugin for ClipboardAux {
    fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
        aviutl2::logger::LogBuilder::new()
            .filter_level(if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            })
            .init();
        Ok(Self {})
    }

    fn register(&mut self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.register_menus::<Self>();
        EDIT_HANDLE.get_or_init(|| registry.create_edit_handle());
    }
}

#[aviutl2::generic::menus]
impl ClipboardAux {
    #[edit(name = "[clipboard.aux2] 貼り付け")]
    fn paste_edit(
        &mut self,
        edit_section: &mut aviutl2::generic::EditSection,
    ) -> aviutl2::AnyResult<()> {
        self.paste_layer(edit_section)
    }

    #[layer(name = "[clipboard.aux2] 貼り付け")]
    fn paste_layer(
        &mut self,
        edit_section: &mut aviutl2::generic::EditSection,
    ) -> aviutl2::AnyResult<()> {
        let mut clipboard =
            arboard::Clipboard::new().context(tr("クリップボードの初期化に失敗しました"))?;
        let edit_handle = EDIT_HANDLE
            .get()
            .expect("EditHandle should be initialized before calling this method");
        let maybe_files = clipboard.get().file_list();
        if let Ok(files) = maybe_files {
            let mut layer = edit_section.info.layer;
            let mut errors = vec![];
            for file in files {
                while !can_place_at(edit_section, layer, edit_section.info.frame)? {
                    layer += 1;
                }
                if !edit_section.is_support_media_file(
                    file.to_string_lossy(),
                    aviutl2::generic::MediaFileSupportMode::ExtensionOnly,
                )? {
                    errors.push((
                        file.to_string_lossy().to_string(),
                        "対応していないファイル形式です",
                    ));
                    continue;
                }
                if edit_section
                    .create_object_from_media_file(
                        file.to_string_lossy(),
                        layer,
                        edit_section.info.frame,
                        None,
                    )
                    .is_err()
                {
                    errors.push((
                        file.to_string_lossy().to_string(),
                        "オブジェクトの作成に失敗しました",
                    ));
                } else {
                    layer += 1;
                }
            }
            if !errors.is_empty() {
                let mut message = tr("以下のファイルの貼り付けに失敗しました:");
                message.push('\n');
                for (file, err) in errors {
                    message.push_str(&format!("- {}: {}\n", file, tr(err)));
                }
                anyhow::bail!(message);
            }
            return Ok(());
        }

        let maybe_img = clipboard.get_image();
        if let Ok(img) = maybe_img {
            let image_dir = get_default_image_dir(edit_handle, edit_section);
            let supports_webp = edit_section.is_support_media_file(
                "z:/test.webp",
                aviutl2::generic::MediaFileSupportMode::ExtensionOnly,
            )?;
            if !image_dir.exists() {
                std::fs::create_dir_all(&image_dir)
                    .context(tr("画像保存用フォルダの作成に失敗しました"))?;
            }
            let file_path = {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let extension = if supports_webp { "webp" } else { "png" };
                image_dir.join(format!("clipboard_{}.{}", timestamp, extension))
            };
            let image = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                img.width as _,
                img.height as _,
                img.bytes.into_owned(),
            )
            .context(tr(
                "クリップボードから取得した画像データの処理に失敗しました",
            ))?;
            image
                .save(&file_path)
                .context(tr("画像ファイルの保存に失敗しました"))?;

            let obj = edit_section.create_object_from_media_file(
                file_path.to_string_lossy(),
                edit_section.info.layer,
                edit_section.info.frame,
                None,
            )?;
            edit_section.focus_object(&obj)?;

            Ok(())
        } else if let Ok(text) = clipboard.get_text() {
            let new_text = edit_section.create_object(
                "テキスト",
                edit_section.info.layer,
                edit_section.info.frame,
                None,
            )?;
            edit_section.set_object_effect_item(&new_text, "テキスト", 0, "テキスト", &text)?;
            edit_section.focus_object(&new_text)?;

            Ok(())
        } else {
            anyhow::bail!(tr("クリップボードに画像またはテキストが見つかりません"));
        }
    }

    #[config(name = "[clipboard.aux2] ファイルの保存先を指定")]
    fn set_aux2_path(&mut self, _hwnd: aviutl2::Win32WindowHandle) -> aviutl2::AnyResult<()> {
        let edit_handle = EDIT_HANDLE
            .get()
            .expect("EditHandle should be initialized before calling this method");
        let current_dir =
            edit_handle.call_edit_section(|edit| get_default_image_dir(edit_handle, edit))?;
        let maybe_new_dir = rfd::FileDialog::new()
            .set_title(tr("保存先フォルダを選択"))
            .set_directory(current_dir)
            .pick_folder();
        if let Some(new_dir) = maybe_new_dir {
            edit_handle
                .call_edit_section(|edit| {
                    let mut proj = edit.get_project_file(edit_handle);
                    proj.set_param_string("save_image_to", new_dir.to_string_lossy().as_ref())
                })?
                .context(tr("保存先の設定に失敗しました"))?;
        }
        Ok(())
    }
}

fn get_default_image_dir(
    edit_handle: &aviutl2::generic::EditHandle,
    edit_section: &mut aviutl2::generic::EditSection,
) -> std::path::PathBuf {
    let proj = edit_section.get_project_file(edit_handle);

    if let Some(save_path) = proj
        .get_param_string("save_image_to")
        .ok()
        .filter(|s| !s.is_empty())
    {
        std::path::PathBuf::from(save_path)
    } else if let Some(proj_dir) = proj
        .get_path()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    {
        proj_dir.join("clipboard.aux2")
    } else {
        let home_dir = dirs::picture_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .expect("Failed to get home directory")
                .join("Pictures")
        });
        home_dir.join("clipboard.aux2")
    }
}

fn can_place_at(
    edit_section: &mut aviutl2::generic::EditSection,
    layer: usize,
    frame: usize,
) -> aviutl2::AnyResult<bool> {
    let next_object = edit_section.find_object_after(layer, frame)?;
    if let Some(obj) = next_object {
        Ok(edit_section.object(&obj).get_layer_frame()?.start > frame)
    } else {
        Ok(true)
    }
}

fn tr(s: &str) -> String {
    aviutl2::config::translate(s).expect("source contains null byte")
}

aviutl2::register_generic_plugin!(ClipboardAux);
