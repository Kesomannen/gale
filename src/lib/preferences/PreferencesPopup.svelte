<script lang="ts">
	import Popup from '../Popup.svelte';
	import PathSetting from './PathSetting.svelte';
	import PreferencesGroup from './PreferencesGroup.svelte';

	export let open = false;
</script>

<Popup title="Preferences" bind:open>
	<PreferencesGroup title="Paths">
		<PathSetting label="Steam executable" name="steam_exe_path" type="exe" />
		<PathSetting label="Download cache directory" name="cache_path" type="dir" />
		<PathSetting label="Data directory" name="data_path" type="dir" />
	</PreferencesGroup>
</Popup>
