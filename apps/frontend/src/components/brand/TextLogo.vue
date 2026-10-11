<template>
	<svg
		xmlns="http://www.w3.org/2000/svg"
		viewBox="0 0 3307 593"
		:class="{ animate: loading }"
		role="img"
		aria-label="ShroudEdit"
	>
		<image
			:href="ShroudEditLogoUrl"
			x="0"
			y="1"
			width="590"
			height="590"
			preserveAspectRatio="xMidYMid meet"
			class="brand-icon"
		/>
		<text
			x="695"
			y="492"
			fill="currentColor"
			style="
				font-family: Inter, Arial, sans-serif;
				font-size: 500px;
				font-weight: 700;
				letter-spacing: -30px;
			"
		>
			ShroudEdit
		</text>
	</svg>
</template>

<script setup lang="ts">
import { ShroudEditLogoUrl } from '@shroudedit/assets'

const loading = useLoading()
</script>

<style lang="scss" scoped>
.animate .brand-icon {
	transform-box: fill-box;
	transform-origin: center;
	animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
	0%,
	100% {
		opacity: 0.72;
		transform: scale(0.96);
	}
	50% {
		opacity: 1;
		transform: scale(1);
	}
}
</style>
