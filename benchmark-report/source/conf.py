import matplotlib
import matplotlib.pyplot as plt

matplotlib.use('agg')
plt.ioff()

project = 'qsdr-benchmarks-report'
copyright = '2024, Daniel Estevez'
author = 'Daniel Estévez'
release = '0.1.0'

extensions = [
    'sphinx.ext.autosectionlabel',
    'matplotlib.sphinxext.plot_directive',
]

templates_path = ['_templates']
exclude_patterns = []

plot_html_show_formats = False
plot_html_show_source_link = False

plot_rcparams = {
     'figure.figsize': (7, 4),
     'savefig.bbox': 'tight',
     'figure.max_open_warning': 0,
}

html_theme = 'furo'
html_static_path = ['_static']
