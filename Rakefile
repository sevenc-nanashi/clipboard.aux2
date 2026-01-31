# frozen_string_literal: true
require "bundler/setup"

desc "ビルドします"
task :build do
  sh "cargo build --release"
end

desc "リリースアセットを作成します"
task :release => [:build] do
  require "zip"
  require "tomlrb"

  version = Tomlrb.load_file("./Cargo.toml")["package"]["version"]
  rm_rf "release" if Dir.exist?("release")
  mkdir "release"
  release_md = File.read("./release.md")
  File.write("./release/README.md", release_md.gsub("{{version}}", version))
  Zip::File.open("./release/clipboard-#{version}.au2pkg.zip", create: true) do |zipfile|
    zipfile.mkdir("Plugin")
    zipfile.add("Plugin/clipboard.aux2", "./target/release/clipboard_aux2.dll")
    zipfile.mkdir("Language")
    Dir.glob("./i18n/*.aul2").each do |lang_file|
      zipfile.add("Language/#{File.basename(lang_file)}", lang_file)
    end
  end

  sh "cargo about generate ./about.hbs -o ./release/THIRD_PARTY_NOTICES.md"
end


desc "./test_environment下にAviUtl2をセットアップし、debugビルドへのシンボリックリンクを作成します"
task :debug_setup do |task, args|
  require "zip"

  unless File.exist?("./test_environment/aviutl2.exe")
    zip_path = "./test_environment/aviutl2_latest.zip"
    mkdir_p("./test_environment") unless Dir.exist?("./test_environment")
    File.open(zip_path, "wb") do |file|
      require "open-uri"
      URI.open(
        "https://api.aviutl2.jp/download?version=latest&type=zip"
      ) { |uri| file.write(uri.read) }
    end
    Zip::File.open(zip_path) do |zip_file|
      zip_file.each do |entry|
        dest_path = File.join("./test_environment", entry.name)
        unless Dir.exist?(File.dirname(dest_path))
          mkdir_p(File.dirname(dest_path))
        end
        rm_rf(dest_path) if File.exist?(dest_path)
        zip_file.extract(entry, dest_path)
      end
    end
    rm(zip_path)
  end

  dest_dir = "./test_environment/data/Plugin"
  language_dir = "./test_environment/data/Language"
  target = "debug"
  FileUtils.mkdir_p(dest_dir) unless Dir.exist?(dest_dir)
  FileUtils.mkdir_p(language_dir) unless Dir.exist?(language_dir)
  ln_s "#{__dir__}/target/#{target}/clipboard_aux2.dll",
      File.join(dest_dir, "clipboard.aux2"),
       force: true
  Dir.glob("./i18n/*.aul2").each do |lang_file|
    ln_s "#{__dir__}/#{lang_file}",
        File.join("./test_environment/data/Language", File.basename(lang_file)),
        force: true
  end
end
