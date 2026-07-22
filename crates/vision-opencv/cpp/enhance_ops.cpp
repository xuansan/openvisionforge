#include "bridge_common.hpp"

#include <cmath>

static cv::Mat equalize_luma(const cv::Mat& image, bool clahe, double clip_limit = 2.0, int tile = 8) {
    if (image.channels() == 1) {
        cv::Mat out;
        if (clahe) {
            auto op = cv::createCLAHE(clip_limit, cv::Size(tile, tile));
            op->apply(image, out);
        } else {
            cv::equalizeHist(image, out);
        }
        return out;
    }

    cv::Mat bgr;
    if (image.channels() == 4) {
        cv::cvtColor(image, bgr, cv::COLOR_BGRA2BGR);
    } else {
        bgr = image;
    }

    cv::Mat ycrcb;
    cv::cvtColor(bgr, ycrcb, cv::COLOR_BGR2YCrCb);
    std::vector<cv::Mat> channels;
    cv::split(ycrcb, channels);
    if (clahe) {
        auto op = cv::createCLAHE(clip_limit, cv::Size(tile, tile));
        op->apply(channels[0], channels[0]);
    } else {
        cv::equalizeHist(channels[0], channels[0]);
    }
    cv::merge(channels, ycrcb);

    cv::Mat out;
    cv::cvtColor(ycrcb, out, cv::COLOR_YCrCb2BGR);
    return out;
}

static std::string op_enhance(const std::vector<std::string>& args) {
    if (args.size() < 4) {
        fail("usage: enhance <mode> <input> <output> [params...]");
    }
    const std::string& mode = args[1];
    cv::Mat image = read_image(args[2]);
    cv::Mat out;

    if (mode == "gamma") {
        if (args.size() < 5) {
            fail("usage: enhance gamma <input> <output> <gamma>");
        }
        double gamma = std::max(std::stod(args[4]), 0.01);
        cv::Mat lut(1, 256, CV_8U);
        for (int i = 0; i < 256; ++i) {
            lut.at<uchar>(i) = cv::saturate_cast<uchar>(std::pow(i / 255.0, gamma) * 255.0);
        }
        cv::LUT(image, lut, out);
    } else if (mode == "hist_equalize") {
        out = equalize_luma(image, false);
    } else if (mode == "clahe") {
        if (args.size() < 6) {
            fail("usage: enhance clahe <input> <output> <clip_limit> <tile_grid_size>");
        }
        double clip_limit = std::max(std::stod(args[4]), 0.1);
        int tile = std::max(std::stoi(args[5]), 1);
        out = equalize_luma(image, true, clip_limit, tile);
    } else if (mode == "contrast_stretch") {
        if (args.size() < 6) {
            fail("usage: enhance contrast_stretch <input> <output> <low> <high>");
        }
        double low = std::stod(args[4]);
        double high = std::stod(args[5]);
        if (low <= 0.0 && high >= 255.0) {
            cv::minMaxLoc(image.reshape(1), &low, &high);
        }
        if (high <= low) {
            out = image.clone();
        } else {
            image.convertTo(out, CV_8U, 255.0 / (high - low), -low * 255.0 / (high - low));
        }
    } else {
        fail("unknown enhance mode: " + mode);
    }

    write_image(args[3], out);
    return metadata_json(out, args[3]);
}
