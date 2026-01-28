const LOCALE = document.querySelector('meta[name="locale"]').content;
const CURRENCY = document.querySelector('meta[name="currency_code"]').content;

document.addEventListener("DOMContentLoaded", (e) => {
    document.querySelectorAll("[data-price]").forEach(e => {
        let priceStr = new Intl.NumberFormat(
            LOCALE || "en-EN",
            { style: "currency", currency: CURRENCY || "usd" }
        ).format(e.dataset.price);
        e.textContent = priceStr;
    });

    // Not needed actually
    document.querySelectorAll("[data-timestamp]").forEach(e => {
        e.textContent = new Date(e.dataset.timestamp * 1000).toLocaleDateString(
            LOCALE || "en-EN",
            {year: "numeric", month: "short", day: "2-digit", weekday: "short", hour: "2-digit", minute: "2-digit"}
        );
    });
});

window.addEventListener('load', function () {
    window.glslCanvases.forEach(c => {
        let s = c.realToCSSPixels;
        let w = c.width;
        let h = c.height;
        c.gl.canvas.width = w * s;
        c.gl.canvas.height = h * s;
    });
});
