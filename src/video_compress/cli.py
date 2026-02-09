import logging
import sys

from PyQt5.QtWidgets import QApplication

from .ui import MainWindow
from .utils import setup_logging


def main() -> None:
    """应用入口点"""
    setup_logging()
    logger = logging.getLogger(__name__)
    logger.info("Starting VideoCompressPro...")

    app = QApplication(sys.argv)
    window = MainWindow()
    window.show()
    sys.exit(app.exec_())


if __name__ == "__main__":
    main()
