window.addEventListener("load", function() {
	document.getElementById("own_url").textContent = document.URL.replace(/\/$/, "");
});
