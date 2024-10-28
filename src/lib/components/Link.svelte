<script lang="ts">
	export let href: string;
</script>

<a
	{href}
	target="_blank"
	rel="noopener noreferrer"
	class="text-accent-400 hover:text-accent-300 hover:underline"
>
	<slot />
</a>
