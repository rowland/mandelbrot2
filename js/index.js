import("../crate/pkg").then(module => {
  var t0 = Date.now();
  module.run();
  console.log(Date.now() - t0);
});
