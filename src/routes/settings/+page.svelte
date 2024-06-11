<script lang="ts">
	import PathPref from '$lib/prefs/PathPref.svelte';
	import LaunchModePref from '$lib/prefs/LaunchModePref.svelte';
	import ZoomLevelPref from '$lib/prefs/ZoomFactorPref.svelte';
	import TogglePref from '$lib/prefs/TogglePref.svelte';
	import { Separator } from 'bits-ui';
	import { currentGame } from '$lib/stores';
</script>

<div class="flex flex-col gap-1 p-6 w-full overflow-y-auto">
	<LaunchModePref />
	<ZoomLevelPref />

	<Separator.Root class="my-2" />

	<PathPref label="Steam executable" key="steam_exe_path" type="file">
		Path to the Steam executable.
	</PathPref>

	<PathPref label="Steam library" key="steam_game_dir" type="dir">
		Path to the Steam game library. This should <b>contain</b> the 'steamapps' directory.
	</PathPref>

	<PathPref
		label="Override game directory"
		key="{$currentGame?.id}_game_dir"
		type="dir"
		canClear={true}
	>
		Path to the {$currentGame?.displayName} game directory.
		Leave empty to use the default Steam library.
	</PathPref>

	<Separator.Root class="my-2" />

	<PathPref label="Data directory" key="data_dir" type="dir">
		Directory where profiles, logs and other app data is stored.
		<br />
		Changing this will move the existing data.
	</PathPref>

	<PathPref label="Temp directory" key="temp_dir" type="dir">
		Directory where temporary files are stored, for example import and export files.
	</PathPref>

	<PathPref label="Download cache directory" key="cache_dir" type="dir">
		Directory where cached mods are stored.
		<br />
		Changing this will move the existing cache.
	</PathPref>

	<TogglePref
		label="Use download cache"
		key="enable_mod_cache"
		disableMessage="This will delete all cached mods. Are you sure?"
	>
		Whether to cache downloaded mods. This speeds up install times and lowers bandwidth usage, but
		can take a considerable amount of disk space.
		<br />
		<b>Warning:</b> Disabling this will delete the existing cache.
	</TogglePref>
</div>
