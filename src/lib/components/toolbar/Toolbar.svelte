<script lang="ts">
	import Updater from './Updater.svelte';
	import Syncer from './Syncer.svelte';
	import ProfilesDropdown from './ProfilesDropdown.svelte';
	import InstallPopover from './InstallPopover.svelte';
	import LaunchButton from './LaunchButton.svelte';
	import GameSelectButton from './GameSelectButton.svelte';
</script>

<div class="bg-primary-900 flex shrink-0 items-center px-3 py-2">
	<LaunchButton />
	<div class="border-primary-700 mx-2 h-full border-r"></div>
	<GameSelectButton />
	<div class="border-primary-700 mx-2 h-full border-r"></div>
	<ProfilesDropdown />
	<Syncer />
	<InstallPopover />
	<Updater />
</div>
