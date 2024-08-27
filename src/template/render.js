const pug = require('pug');
const fs = require('fs');

// Function to render Pug template to HTML
function renderTemplate(templatePath, outputPath, data) {
  const compiledFunction = pug.compileFile(templatePath);
  const html = compiledFunction(data);

  fs.writeFileSync(outputPath, html);
}

renderTemplate('views/index.pug', '@/templates/index.html', {});