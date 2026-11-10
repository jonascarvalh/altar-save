// Alternância de tema claro/escuro. O tema inicial é aplicado no <head> (index.html).
(function () {
  var KEY = "altar-save-site:theme";
  var root = document.documentElement;
  var button = document.getElementById("theme-toggle");

  function label() {
    button.setAttribute(
      "aria-label",
      root.dataset.theme === "light" ? "Mudar para o tema escuro" : "Mudar para o tema claro",
    );
  }

  label();
  button.addEventListener("click", function () {
    var next = root.dataset.theme === "light" ? "dark" : "light";
    root.dataset.theme = next;
    try {
      localStorage.setItem(KEY, next);
    } catch (e) {
      // Sem armazenamento: o tema vale só para esta visita.
    }
    label();
  });
})();
