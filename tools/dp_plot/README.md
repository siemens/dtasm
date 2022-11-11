# Dp Plot

Simple Jupyter notebook for animating double pendulum motion as simulated by some of the `dtasm` examples. A recent version of Python 3 with `venv` package is needed.

Steps: 
1. Create and activate a virtual environment: 
   ```
   python -mvenv .venv
   source .venv/bin/activate
   (or on Windows: .venv/Scripts/activate.bat)
   ```
2. Install Jupyter, numpy and matplotlib:
   `python -mpip install -r requirements.txt`
3. Run Jupyterlab:
   `jupyter lab`
4. Open your browser at the displayed url, open `double_pendulum_animation.ipynb` notebook, paste output of the examples into `result_csv` variable and run all cells.