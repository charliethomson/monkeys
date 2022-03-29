// Paste into http://shakespeare.mit.edu/hamlet/full.html to get a csv file of all the words in hamlet :)
(() => {
  const words = Object.keys(
    [...document.querySelectorAll("blockquote")]
      .map((node) => [...node.children].map((child) => child.textContent))
      .reduce((a, c) => [...a, ...c], [])
      .reduce((a, c) => [...a, c.split(" ")], [])
      .reduce((a, c) => [...a, ...c], [])
      .map((word) => (word?.match(/\w/g) ?? []).join("").toLowerCase())
      .filter((w) => w?.trim?.()?.length ?? 0 >= 3)
      .reduce((a, c) => {
        c.length !== 0 ? (a[c] ? a[c]++ : (a[c] = 1)) : void 0;
        return a;
      }, {})
  );

  const wordsCsv = words.join(",");
  const el = document.createElement("a");
  el.setAttribute(
    "href",
    "data:text/plain;charset=utf-8," + encodeURIComponent(wordsCsv)
  );
  el.setAttribute("download", "hamletWords.csv");
  el.click();
})();
