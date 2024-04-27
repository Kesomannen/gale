<script lang="ts">
	import Popup from '../Popup.svelte';
	import PathSetting from './PathPref.svelte';
	import LaunchModePref from './LaunchModePref.svelte';

	export let open = false;
</script>

<Popup title="Preferences" bind:open>
	<div class="flex flex-col gap-1 pt-2">
		<LaunchModePref />
		<PathSetting label="Steam executable" key="steam_exe_path" type="exe" />
		<PathSetting label="Download cache directory" key="cache_dir" type="dir" />
		<PathSetting label="Data directory" key="data_dir" type="dir" />
		<PathSetting label="Temp directory" key="temp_dir" type="dir" />
	</div>
</Popup>
