#include <iostream>
#include <opencv2/opencv.hpp>

int main(int argc, char** argv) {
    const std::string input = argc > 1 ? argv[1] : "demo/input/d34084cc1f765c877abcdf2d515cacf3.jpg";
    const std::string output = argc > 2 ? argv[2] : "demo/output/opencv5_binary.png";

    cv::Mat image = cv::imread(input, cv::IMREAD_COLOR);
    if (image.empty()) {
        std::cerr << "failed to read image: " << input << std::endl;
        return 1;
    }

    cv::Mat gray;
    cv::cvtColor(image, gray, cv::COLOR_BGR2GRAY);

    cv::Mat binary;
    cv::threshold(gray, binary, 128, 255, cv::THRESH_BINARY);

    const int total = binary.rows * binary.cols;
    const int white = cv::countNonZero(binary);
    const int black = total - white;

    if (!cv::imwrite(output, binary)) {
        std::cerr << "failed to write image: " << output << std::endl;
        return 1;
    }

    std::cout << "OpenCV version: " << CV_VERSION << std::endl;
    std::cout << "resolution: " << image.cols << "x" << image.rows << std::endl;
    std::cout << "black ratio: " << static_cast<double>(black) / total << std::endl;
    std::cout << "white ratio: " << static_cast<double>(white) / total << std::endl;
    std::cout << "output: " << output << std::endl;
    return 0;
}
